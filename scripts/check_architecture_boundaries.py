#!/usr/bin/env python3
"""架构边界检查脚本

检查以下架构约束：
1. 任务生命周期桩函数不应在 Rust db_bridge.rs 中注册
2. TaskApplicationService 中不应有 `let _ = self.db` 的使用
3. 迁移的模式不应使用 py_start_batch_download

用法：
    python scripts/check_architecture_boundaries.py

退出码：
    0 - 所有检查通过
    1 - 发现架构违规
"""

import re
import sys
from pathlib import Path

# 项目根目录
PROJECT_ROOT = Path(__file__).resolve().parent.parent

# Phase Two 临时例外：每项 (检查名, 模式描述, 关联 issue, 说明)
# 完成对应 issue 后必须从此处删除并让检查变为硬失败
ALLOWED_EXCEPTIONS = []


def _is_allowed_exception(check_name):
    for name, _desc, issue, note in ALLOWED_EXCEPTIONS:
        if name == check_name:
            return issue, note
    return None, None

def check_task_lifecycle_stubs():
    """检查 Rust db_bridge.rs 中是否注册了任务生命周期桩函数"""
    db_bridge_path = PROJECT_ROOT / "src-tauri" / "src" / "python" / "db_bridge.rs"

    if not db_bridge_path.exists():
        print(f"✓ 文件不存在: {db_bridge_path}")
        return True

    content = db_bridge_path.read_text(encoding="utf-8")

    # 检查是否有任务生命周期闭包注册（setattr 调用）
    # 只检查实际的注册代码，不检查注释
    forbidden_patterns = [
        r'setattr.*_create_task',
        r'setattr.*_update_task_status',
        r'setattr.*_create_task_item',
        r'setattr.*_update_task_item_status',
    ]

    violations = []
    for pattern in forbidden_patterns:
        if re.search(pattern, content):
            violations.append(f"发现禁止的任务生命周期桩函数注册: {pattern}")

    if violations:
        print("✗ 架构违规：Rust db_bridge.rs 中仍注册了任务生命周期桩函数")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ Rust db_bridge.rs 中未注册任务生命周期桩函数")
    return True

def check_silent_db_errors():
    """检查 TaskApplicationService 中是否有静默忽略 DB 错误的模式"""
    service_path = PROJECT_ROOT / "src-tauri" / "src" / "tasks" / "service.rs"

    if not service_path.exists():
        print(f"✓ 文件不存在: {service_path}")
        return True

    content = service_path.read_text(encoding="utf-8")
    lines = content.split('\n')

    violations = []

    # 检查关键 DB 操作是否被 warn! 忽略
    critical_operations = [
        'create_task_item',
        'update_task_item_status',
        'save_batch_results',
        'update_task_counts',
    ]

    for i, line in enumerate(lines, 1):
        # 检查 let _ = self.db 模式
        if 'let _ = self.db' in line:
            violations.append(f"第{i}行: 使用 `let _ = self.db` 静默忽略错误")

        # 检查关键操作是否只是 warn! 而没有返回错误
        for op in critical_operations:
            if op in line and 'warn!' in line and i > 0:
                # 检查前一行是否是 if let Err(e) = self.db.xxx
                prev_line = lines[i-2] if i >= 2 else ''
                if f'self.db.{op}' in prev_line and 'if let Err' in prev_line:
                    # 检查下一行是否是 return Err
                    next_line = lines[i] if i < len(lines) else ''
                    if 'return Err' not in next_line:
                        violations.append(f"第{i}行: {op} 失败只是 warn!，应该返回错误")

    if violations:
        print("✗ 架构违规：TaskApplicationService 中有静默忽略 DB 错误的模式")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ TaskApplicationService 中未发现静默忽略 DB 错误的模式")
    return True

def check_batch_download_fallback():
    """检查是否还有对 py_start_batch_download 的引用"""
    search_dirs = [
        PROJECT_ROOT / "src",
        PROJECT_ROOT / "src-tauri" / "src",
        PROJECT_ROOT / "backend",
        PROJECT_ROOT / "core",
    ]

    violations = []
    for search_dir in search_dirs:
        if not search_dir.exists():
            continue

        for file_path in search_dir.rglob("*"):
            if not file_path.is_file():
                continue

            # 只检查代码文件
            if file_path.suffix not in ['.py', '.rs', '.ts', '.tsx']:
                continue

            try:
                content = file_path.read_text(encoding="utf-8")
                if 'py_start_batch_download' in content:
                    violations.append(f"{file_path.relative_to(PROJECT_ROOT)}")
            except (UnicodeDecodeError, PermissionError):
                continue

    if violations:
        print("✗ 架构违规：发现对 py_start_batch_download 的引用")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ 未发现对 py_start_batch_download 的引用")
    return True

def check_py_download_video():
    """检查是否还有对 py_download_video 的引用（前端 invoke 调用）"""
    search_dirs = [
        PROJECT_ROOT / "src",
    ]

    violations = []
    for search_dir in search_dirs:
        if not search_dir.exists():
            continue

        for file_path in search_dir.rglob("*"):
            if not file_path.is_file():
                continue

            # 只检查 TypeScript 文件
            if file_path.suffix not in ['.ts', '.tsx']:
                continue

            try:
                content = file_path.read_text(encoding="utf-8")
                # 检查 invoke("py_download_video") 调用
                if 'invoke("py_download_video")' in content or "invoke('py_download_video')" in content:
                    violations.append(f"{file_path.relative_to(PROJECT_ROOT)}: invoke(\"py_download_video\")")
            except (UnicodeDecodeError, PermissionError):
                continue

    if violations:
        print("✗ 架构违规：前端代码中发现对 py_download_video 的 invoke 调用")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ 前端代码中未发现对 py_download_video 的 invoke 调用")
    return True

def check_redline_shims():
    """检查 RED LINE 文件的 shim 是否正确（5行 sys.modules 别名模式）"""
    # Each shim file should reference a specific module path via sys.modules alias
    shim_files = {
        "core/crawler.py": "crawler_engine.crawler",
        "core/filter.py": "crawler_engine.filter",
        "core/api.py": "crawler_engine.api",
        "core/py_bridge.py": "bridge.py_bridge",
        "core/handler.py": "bridge.handler",
        "core/tauri_bridge.py": "bridge.events",
        "core/db_bridge.py": "bridge.db_bridge",
        "core/downloader.py": "download.downloader",
    }

    violations = []
    for shim_path, expected_ref in shim_files.items():
        full_path = PROJECT_ROOT / shim_path
        if not full_path.exists():
            violations.append(f"{shim_path} 不存在（应该是 shim 文件）")
            continue

        content = full_path.read_text(encoding="utf-8")
        lines = content.strip().split("\n")

        # Shims should be short (≤15 lines) and reference the real module
        if len(lines) > 15:
            violations.append(f"{shim_path} shim 过长 ({len(lines)}行)，可能不是 shim")
        if expected_ref not in content and "import *" not in content:
            violations.append(f"{shim_path} shim 未引用 {expected_ref}")

    if violations:
        print("✗ 架构违规：RED LINE shim 文件检查失败")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ RED LINE shim 文件正确（所有旧路径均指向新位置）")
    return True


def check_models_split():
    """检查 models.py 已拆分为 models/ 子包"""
    models_dir = PROJECT_ROOT / "core" / "models"
    models_file = PROJECT_ROOT / "core" / "models.py"

    violations = []
    if not models_dir.is_dir():
        violations.append("core/models/ 目录不存在")
    else:
        required_files = ["__init__.py", "requests.py", "config.py", "download.py", "responses.py"]
        for f in required_files:
            if not (models_dir / f).exists():
                violations.append(f"core/models/{f} 缺失")

    if models_file.exists():
        content = models_file.read_text(encoding="utf-8")
        if len(content.split("\n")) > 15:
            violations.append("core/models.py 仍存在且不是 shim（应该删除或改为短 shim）")

    if violations:
        print("✗ 架构违规：models 拆分不完整")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ models/ 子包结构正确")
    return True


def check_frontend_py_start_download():
    """检查前端 src/ 不得 invoke py_start_download"""
    search_dir = PROJECT_ROOT / "src"
    violations = []
    for file_path in search_dir.rglob("*"):
        if not file_path.is_file() or file_path.suffix not in ('.ts', '.tsx'):
            continue
        try:
            content = file_path.read_text(encoding="utf-8")
            if 'invoke("py_start_download")' in content or "invoke('py_start_download')" in content:
                violations.append(file_path.relative_to(PROJECT_ROOT))
        except (UnicodeDecodeError, PermissionError):
            continue

    if violations:
        print("✗ 架构违规：前端代码中发现 invoke(\"py_start_download\") 调用")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ 前端代码中未发现对 py_start_download 的 invoke 调用")
    return True


def check_py_start_download_command_registration():
    """检查 Tauri lib.rs 的 generate_handler! 中不得注册 py_start_download"""
    lib_path = PROJECT_ROOT / "src-tauri" / "src" / "lib.rs"
    if not lib_path.exists():
        print(f"✓ 文件不存在: {lib_path}")
        return True

    content = lib_path.read_text(encoding="utf-8")
    for i, line in enumerate(content.split('\n'), 1):
        if 'py_start_download' in line:
            print(f"✗ 架构违规：lib.rs:{i} 仍注册了 py_start_download")
            return False

    print("✓ lib.rs 中未注册 py_start_download command")
    return True


def check_py_test_emit_registration():
    """检查 Tauri 不得注册 py_test_emit command"""
    search_dir = PROJECT_ROOT / "src-tauri" / "src"
    violations = []
    for file_path in search_dir.rglob("*.rs"):
        if not file_path.is_file():
            continue
        try:
            content = file_path.read_text(encoding="utf-8")
            for i, line in enumerate(content.split('\n'), 1):
                if 'py_test_emit' in line:
                    violations.append(f"{file_path.relative_to(PROJECT_ROOT)}:{i}")
        except (UnicodeDecodeError, PermissionError):
            continue

    if violations:
        issue, note = _is_allowed_exception("py_test_emit_registration")
        if note:
            print(f"⚠ 临时例外（允许）：{note} [TODO {issue}]")
            for v in violations:
                print(f"  - {v}")
            return True
        print("✗ 架构违规：Tauri 代码中仍注册了 py_test_emit command")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ Tauri 代码中未发现 py_test_emit command")
    return True


def check_python_db_writes():
    """检查 Python 普通下载路径不得写普通任务 DB；live 写入受控"""
    search_dirs = [
        PROJECT_ROOT / "core" / "download",
        PROJECT_ROOT / "core" / "task",
        PROJECT_ROOT / "backend",
    ]
    excluded_files = {
        PROJECT_ROOT / "core" / "db.py",
        PROJECT_ROOT / "core" / "bridge" / "db_bridge.py",
        PROJECT_ROOT / "core" / "bridge" / "__init__.py",
        PROJECT_ROOT / "core" / "__init__.py",
    }
    live_manager_path = (PROJECT_ROOT / "core" / "task" / "live_manager.py").resolve()

    hard_violations = []
    allowed_violations = []

    for search_dir in search_dirs:
        if not search_dir.exists():
            continue
        for file_path in search_dir.rglob("*.py"):
            if not file_path.is_file() or file_path in excluded_files:
                continue
            try:
                content = file_path.read_text(encoding="utf-8")
            except (UnicodeDecodeError, PermissionError):
                continue

            for i, line in enumerate(content.split('\n'), 1):
                if 'save_live_record(' in line:
                    if file_path.resolve() == live_manager_path:
                        allowed_violations.append(f"{file_path.relative_to(PROJECT_ROOT)}:{i} 调用了 save_live_record")
                    else:
                        hard_violations.append(f"{file_path.relative_to(PROJECT_ROOT)}:{i} 调用了 save_live_record")

    for v in allowed_violations:
        issue, note = _is_allowed_exception("python_db_writes")
        if note:
            print(f"⚠ 临时例外（允许）：{note} [TODO {issue}]")
            print(f"  - {v}")
        else:
            hard_violations.append(v)

    if hard_violations:
        print("✗ 架构违规：Python 业务层绕过了 Rust 直接写 DB")
        for v in hard_violations:
            print(f"  - {v}")
        return False

    if not allowed_violations:
        print("✓ Python 业务层中未发现直接 DB 写入")
    return True


def check_api_types_rust_owned_duplicates():
    """检查 api-types.ts 不得定义 Rust-owned 基础类型"""
    file_path = PROJECT_ROOT / "src" / "lib" / "api-types.ts"
    if not file_path.exists():
        print(f"✓ 文件不存在: {file_path}")
        return True

    content = file_path.read_text(encoding="utf-8")
    rust_owned_types = ["DownloadMode", "TaskStatus", "TaskEventType", "TaskEvent", "ErrorCode"]
    pattern = re.compile(r'export\s+(type|interface|enum)\s+(' + '|'.join(rust_owned_types) + r')\b')

    violations = []
    for i, line in enumerate(content.split('\n'), 1):
        m = pattern.search(line)
        if m:
            violations.append(f"{file_path.relative_to(PROJECT_ROOT)}:{i} 定义了 Rust-owned 类型 {m.group(2)}")

    if violations:
        issue, note = _is_allowed_exception("api_types_rust_owned_duplicates")
        if note:
            print(f"⚠ 临时例外（允许）：{note} [TODO {issue}]")
            for v in violations:
                print(f"  - {v}")
            return True
        print("✗ 架构违规：api-types.ts 重复定义了 Rust-owned 基础类型")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ api-types.ts 未定义 Rust-owned 基础类型")
    return True


def main():
    """运行所有架构边界检查"""
    print("=" * 60)
    print("架构边界检查")
    print("=" * 60)

    checks = [
        ("任务生命周期桩函数", check_task_lifecycle_stubs),
        ("静默 DB 错误", check_silent_db_errors),
        ("批量下载回退", check_batch_download_fallback),
        ("py_download_video 引用", check_py_download_video),
        ("RED LINE shim 文件", check_redline_shims),
        ("models 子包拆分", check_models_split),
        ("前端 py_start_download", check_frontend_py_start_download),
        ("py_start_download command 注册", check_py_start_download_command_registration),
        ("py_test_emit 注册", check_py_test_emit_registration),
        ("Python 直接 DB 写入", check_python_db_writes),
        ("api-types 重复定义", check_api_types_rust_owned_duplicates),
    ]

    all_passed = True
    for name, check_fn in checks:
        print(f"\n检查: {name}")
        print("-" * 40)
        if not check_fn():
            all_passed = False

    print("\n" + "=" * 60)
    if all_passed:
        print("✓ 所有架构边界检查通过")
    else:
        print("✗ 发现架构违规，请修复后重试")

    print("\n" + "=" * 60)
    print("当前生效的临时例外（完成对应 issue 后删除）")
    print("-" * 60)
    for name, desc, issue, note in ALLOWED_EXCEPTIONS:
        print(f"  [{issue}] {desc}")
        print(f"          {note}")
    print("=" * 60)

    return 0 if all_passed else 1

if __name__ == "__main__":
    sys.exit(main())
