#!/usr/bin/env python3
"""CI 架构边界守卫 — 阻断架构回归

检查项:
1. RED LINE 文件完整性 — 禁止修改内部逻辑的文件未被改动核心代码
2. 禁止的 import 路径 — 防止跨层违规依赖
3. 包结构一致性 — 子包 __init__.py 重导出完整性
4. 死模块引用 — backend/ 目录已被删除，不应有 import

用法:
    python scripts/check_architecture.py          # 检查模式
    python scripts/check_architecture.py --ci     # CI 模式（非零退出码阻断）
"""

import ast
import os
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
CORE_DIR = PROJECT_ROOT / "core"
SRC_TAURI_DIR = PROJECT_ROOT / "src-tauri" / "src"

# ============================================================
# 1. RED LINE 文件清单
# ============================================================

RED_LINE_FILES = [
    "core/crawler_engine/crawler.py",
    "core/crawler_engine/filter.py",
    "core/crawler_engine/signature/abogus.py",
    "core/crawler_engine/signature/xbogus.py",
    "core/crawler_engine/signature/fingerprint.py",
    "core/crawler_engine/tokens/token_manager.py",
]

# ============================================================
# 2. 禁止的 import 路径
# ============================================================

FORBIDDEN_IMPORTS = {
    # backend/ 已删除，禁止任何引用（排除本脚本自身）
    "from backend.": "backend/ 目录已删除，使用 core.task 替代",
    "import backend.": "backend/ 目录已删除，使用 core.task 替代",
}

# 子包内部应使用相对 import
RELATIVE_IMPORT_REQUIRED_IN = ["core/crawler_engine/services"]

# ============================================================
# 3. 核心 __init__.py 要求
# ============================================================

EXPECTED_INIT_FILES = [
    "core/__init__.py",
    "core/crawler_engine/__init__.py",
    "core/bridge/__init__.py",
    "core/download/__init__.py",
    "core/task/__init__.py",
    "core/utils/__init__.py",
]


def check_red_line_files() -> list[str]:
    """检查 RED LINE 文件是否存在且语法有效"""
    errors = []
    for rel_path in RED_LINE_FILES:
        full_path = PROJECT_ROOT / rel_path
        if not full_path.exists():
            errors.append(f"[RED LINE] 文件缺失: {rel_path}")
            continue

        with open(full_path) as f:
            content = f.read()

        try:
            ast.parse(content)
        except SyntaxError as e:
            errors.append(f"[RED LINE] 语法错误 {rel_path}: {e}")
    return errors


def check_forbidden_imports() -> list[str]:
    """扫描所有 Python 文件，检测禁止的 import"""
    errors = []
    for py_file in PROJECT_ROOT.rglob("*.py"):
        if "__pycache__" in str(py_file) or ".venv" in str(py_file):
            continue
        rel = py_file.relative_to(PROJECT_ROOT)

        # 跳过本脚本自身
        if rel.parts[0] == "scripts":
            continue

        try:
            with open(py_file) as f:
                content = f.read()
        except Exception:
            continue

        for forbidden, reason in FORBIDDEN_IMPORTS.items():
            if forbidden in content:
                errors.append(f"[FORBIDDEN] {rel}: 包含禁止引用 '{forbidden}' — {reason}")
    return errors


def check_init_files() -> list[str]:
    """检查必需的 __init__.py 是否存在且有内容"""
    errors = []
    for rel_path in EXPECTED_INIT_FILES:
        full_path = PROJECT_ROOT / rel_path
        if not full_path.exists():
            errors.append(f"[INIT] 缺少: {rel_path}")
            continue
        size = full_path.stat().st_size
        if size < 10:  # 不能只是空文件
            errors.append(f"[INIT] 内容过少: {rel_path} ({size} bytes)")
    return errors


def check_relative_imports() -> list[str]:
    """检查子包内部是否使用相对 import"""
    errors = []
    for rel_dir in RELATIVE_IMPORT_REQUIRED_IN:
        dir_path = PROJECT_ROOT / rel_dir
        if not dir_path.exists():
            continue
        for py_file in dir_path.rglob("*.py"):
            if py_file.name == "__init__.py":
                continue
            rel = py_file.relative_to(PROJECT_ROOT)
            with open(py_file) as f:
                content = f.read()
            # 检测跨子包的绝对 import（应使用相对 import）
            for line in content.split("\n"):
                stripped = line.strip()
                if stripped.startswith(f"from {rel_dir.replace('/', '.')}") and "import" in stripped:
                    errors.append(
                        f"[IMPORT] {rel}: 同包子包应使用相对 import，而非: {stripped}"
                    )
    return errors


def main():
    ci_mode = "--ci" in sys.argv
    all_errors = []

    all_errors.extend(check_red_line_files())
    all_errors.extend(check_forbidden_imports())
    all_errors.extend(check_init_files())
    all_errors.extend(check_relative_imports())

    if all_errors:
        print(f"[ARCH CHECK] {len(all_errors)} 个架构违规:")
        for err in all_errors:
            print(f"  ✗ {err}")
        if ci_mode:
            sys.exit(1)
    else:
        print("[ARCH CHECK] ✓ 架构边界完整，无违规")

    if ci_mode:
        sys.exit(0)


if __name__ == "__main__":
    main()
