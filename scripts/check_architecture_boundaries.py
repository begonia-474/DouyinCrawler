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
        return 0
    else:
        print("✗ 发现架构违规，请修复后重试")
        return 1

if __name__ == "__main__":
    sys.exit(main())
