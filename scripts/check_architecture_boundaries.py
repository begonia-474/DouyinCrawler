#!/usr/bin/env python3
"""架构边界检查脚本 — Issue 10 强化版

检查以下架构约束：
1. Tauri command registry 不存在非音乐 Python download/live execution commands
2. Rust Python handler/re-exports 不存在旧 execution wrapper
3. Python handler/task/bridge 不写 download_tasks、task_items、live_records
4. Python 不发非音乐 task/live lifecycle event
5. Frontend download/record action 只调用 Rust-owned commands
6. 唯一允许的 Python download execution 例外是 py_download_music 及其最小调用链
7. guard 扫描当前真实文件路径，并有一个会故意触发违规的脚本自测或 fixture

用法：
    python scripts/check_architecture_boundaries.py

退出码：
    0 - 所有检查通过
    1 - 发现架构违规
"""

import io
import re
import sys
import tokenize
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent


# ============================================================
# 配置
# ============================================================

# 允许的 Python download execution 例外（精确最小集）
MUSIC_ALLOWLIST = {
    "py_download_music",
}

# Python execution exports 黑名单（不得出现在 Rust handler/mod/py_bridge/__init__）
FORBIDDEN_PYTHON_EXECUTION_EXPORTS = {
    "download_video",
    "download_batch",
    "start_download",
    "start_live_record",
    "stop_live_record",
    "get_live_status",
    "get_batch_status",
}

# 前端不得直接 invoke 的旧 Python execution 命令
FORBIDDEN_FRONTEND_PYTHON_COMMANDS = {
    "py_download_video",
    "py_download_batch",
    "py_start_download",
    "py_start_live_record",
    "py_stop_live_record",
    "py_get_live_status",
    "py_get_batch_status",
}

# Python 不得写入的 DB 表（硬编码表名检查）
FORBIDDEN_DB_WRITES = {
    "download_tasks",
    "task_items",
    "live_records",
}

FORBIDDEN_SERVICE_EFFECT_METHODS = {
    "handle_one_video",
    "handle_user_post",
    "handle_user_like",
    "handle_user_mix",
    "handle_collects_video",
    "handle_user_collection",
    "handle_live_record",
}

MUSIC_EFFECT_ALLOWLIST = {
    "py_download_music",
    "download_music",
    "download_music_batch",
    "handle_download_music",
}


def _is_python_execution_command(name: str) -> bool:
    """Return whether a py_* command represents a file/task execution boundary."""
    if name in MUSIC_ALLOWLIST:
        return False
    if name in FORBIDDEN_FRONTEND_PYTHON_COMMANDS:
        return True
    return any(marker in name for marker in ("download", "record", "execute")) or name.startswith(
        ("py_start_", "py_stop_")
    )


def find_python_execution_commands(content: str) -> set[str]:
    """Find forbidden Python-backed execution commands in Rust/TypeScript source."""
    commands = set(re.findall(r"\bpy_[A-Za-z0-9_]+\b", content))
    return {name for name in commands if _is_python_execution_command(name)}


def _is_python_effect_export(name: str) -> bool:
    if name in MUSIC_EFFECT_ALLOWLIST:
        return False
    if name in FORBIDDEN_PYTHON_EXECUTION_EXPORTS:
        return True
    if name in FORBIDDEN_SERVICE_EFFECT_METHODS:
        return True
    return any(marker in name for marker in ("download", "record", "execute"))


def find_python_effect_exports(
    content: str, allowed_names: set[str] | None = None
) -> set[str]:
    """Find forbidden Python/Rust function definitions that expose effects."""
    allowed_names = allowed_names or set()
    names = set(
        re.findall(
            r"\b(?:async\s+def|def|pub\s+(?:async\s+)?fn)\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(",
            content,
        )
    )
    return {
        name
        for name in names
        if name not in allowed_names and _is_python_effect_export(name)
    }


def find_python_reexport_effects(content: str) -> set[str]:
    """Find effect names re-exported from Rust's python::handler module."""
    blocks = re.findall(r"pub\s+use\s+handler::\{(.*?)\};", content, flags=re.DOTALL)
    names: set[str] = set()
    for block in blocks:
        names.update(re.findall(r"\b[a-z_][a-z0-9_]*\b", block))
    return {name for name in names if _is_python_effect_export(name)}


def find_forbidden_db_refs(content: str) -> set[str]:
    """Find task/item/live table references forbidden in Python-owned code."""
    content = _python_code_without_comments_and_docstrings(content)
    return {table for table in FORBIDDEN_DB_WRITES if table in content}


def find_forbidden_event_calls(content: str) -> set[str]:
    """Find Python lifecycle event emission calls."""
    content = _python_code_without_comments_and_docstrings(content)
    violations = set()
    if re.search(r"\bemit\s*\(", content):
        violations.add("emit(")
    if "broadcast_" in content:
        violations.add("broadcast_")
    return violations


def _python_code_without_comments_and_docstrings(content: str) -> str:
    """Remove Python comments/docstrings while preserving executable string literals."""
    output = []
    previous_type = tokenize.INDENT
    try:
        tokens = tokenize.generate_tokens(io.StringIO(content).readline)
        for token_type, token_text, _start, _end, _line in tokens:
            if token_type == tokenize.COMMENT:
                continue
            if token_type == tokenize.STRING and previous_type in {
                tokenize.INDENT,
                tokenize.NEWLINE,
            }:
                continue
            output.append(token_text)
            if token_type not in {
                tokenize.NL,
                tokenize.ENCODING,
                tokenize.COMMENT,
            }:
                previous_type = token_type
    except (IndentationError, tokenize.TokenError):
        return content
    return " ".join(output)


# ============================================================
# 检查函数
# ============================================================

def check_no_legacy_python_execution_in_command_registry():
    """检查 Tauri lib.rs 的 generate_handler! 中不得注册非音乐 Python execution commands"""
    lib_path = PROJECT_ROOT / "src-tauri" / "src" / "lib.rs"
    if not lib_path.exists():
        print(f"✓ 文件不存在: {lib_path}")
        return True

    content = lib_path.read_text(encoding="utf-8")
    violations = [
        f"lib.rs: 注册了禁止的 command `{cmd}`"
        for cmd in sorted(find_python_execution_commands(content))
    ]

    if violations:
        print("✗ Tauri command registry 中包含非音乐 Python execution commands")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ Tauri command registry 不含非音乐 Python execution commands")
    return True


def check_no_legacy_python_execution_in_handler():
    """检查 Rust handler.rs 中不得导出旧 execution wrapper"""
    handler_path = PROJECT_ROOT / "src-tauri" / "src" / "python" / "handler.rs"
    if not handler_path.exists():
        print(f"✓ 文件不存在: {handler_path}")
        return True

    content = handler_path.read_text(encoding="utf-8")
    violations = [
        f"handler.rs: 导出禁止的执行函数 `{export}`"
        for export in sorted(find_python_effect_exports(content))
    ]

    if violations:
        print("✗ Rust handler.rs 导出了旧 execution wrapper")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ Rust handler.rs 未导出旧 execution wrapper")
    return True


def check_no_legacy_execution_in_mod_reexport():
    """检查 Rust python/mod.rs re-export 中不得包含旧 execution"""
    mod_path = PROJECT_ROOT / "src-tauri" / "src" / "python" / "mod.rs"
    if not mod_path.exists():
        print(f"✓ 文件不存在: {mod_path}")
        return True

    content = mod_path.read_text(encoding="utf-8")
    violations = [
        f"mod.rs: re-export 禁止的执行函数 `{export}`"
        for export in sorted(find_python_reexport_effects(content))
    ]

    if violations:
        print("✗ Rust python/mod.rs re-export 中包含旧 execution")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ Rust python/mod.rs re-export 不含旧 execution")
    return True


def check_no_python_db_writes():
    """检查 Python 业务层不写禁止的 DB 表

    覆盖范围：
    - core/download, core/task, core/bridge, core/crawler_engine/services
    - 包括 db_bridge.py，防止重新引入 task/item/live 写入
    - __init__.py 是纯 re-export，可排除
    """
    search_dirs = [
        PROJECT_ROOT / "core" / "db.py",
        PROJECT_ROOT / "core" / "download",
        PROJECT_ROOT / "core" / "task",
        PROJECT_ROOT / "core" / "bridge",
        PROJECT_ROOT / "core" / "crawler_engine" / "services",
    ]
    excluded_files = {
        PROJECT_ROOT / "core" / "bridge" / "__init__.py",
        PROJECT_ROOT / "core" / "__init__.py",
    }

    violations = []
    for search_path in search_dirs:
        if not search_path.exists():
            continue
        candidates = [search_path] if search_path.is_file() else search_path.rglob("*.py")
        for file_path in candidates:
            if not file_path.is_file() or file_path in excluded_files:
                continue
            try:
                content = file_path.read_text(encoding="utf-8")
            except (UnicodeDecodeError, PermissionError):
                continue

            for table in sorted(find_forbidden_db_refs(content)):
                violations.append(
                    f"{file_path.relative_to(PROJECT_ROOT)}: 包含禁止 DB 表名 `{table}`"
                )

    if violations:
        print("✗ Python 业务层绕过了 Rust 直接写禁止的 DB 表")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ Python 业务层未发现禁止的 DB 写入")
    return True


def check_no_python_non_music_events():
    """检查 Python 业务层不发非音乐 task/live lifecycle event（emit 调用）

    覆盖范围：
    - core/download, core/task, core/bridge, core/crawler_engine/services
    """
    search_dirs = [
        PROJECT_ROOT / "core" / "download",
        PROJECT_ROOT / "core" / "task",
        PROJECT_ROOT / "core" / "bridge",
        PROJECT_ROOT / "core" / "crawler_engine" / "services",
    ]

    violations = []
    for search_dir in search_dirs:
        if not search_dir.exists():
            continue
        for file_path in search_dir.rglob("*.py"):
            if not file_path.is_file():
                continue
            try:
                content = file_path.read_text(encoding="utf-8")
            except (UnicodeDecodeError, PermissionError):
                continue

            for marker in sorted(find_forbidden_event_calls(content)):
                violations.append(
                    f"{file_path.relative_to(PROJECT_ROOT)}: 禁止的事件发射 `{marker}`"
                )

    if violations:
        print("✗ Python 业务层发送了禁止的 task/live lifecycle event")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ Python 业务层未发送非音乐 lifecycle event")
    return True


def check_frontend_uses_rust_commands():
    """检查前端 download/record action 只调用 Rust-owned commands"""
    search_dir = PROJECT_ROOT / "src"

    violations = []
    for file_path in search_dir.rglob("*"):
        if not file_path.is_file() or file_path.suffix not in ('.ts', '.tsx'):
            continue
        try:
            content = file_path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, PermissionError):
            continue

        for cmd in sorted(find_python_execution_commands(content)):
            violations.append(
                f"{file_path.relative_to(PROJECT_ROOT)}: 调用禁止的 Python 命令 `{cmd}`"
            )

    if violations:
        print("✗ 前端调用了禁止的 Python execution command")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ 前端 download/record action 仅调用 Rust-owned commands")
    return True


def check_no_python_effect_exports():
    """Check Python facade/handler/services for non-music execution functions."""
    targets = [
        PROJECT_ROOT / "core" / "bridge" / "py_bridge.py",
        PROJECT_ROOT / "core" / "bridge" / "handler.py",
        PROJECT_ROOT / "core" / "crawler_engine" / "services",
    ]
    violations = []
    for target in targets:
        if not target.exists():
            continue
        candidates = [target] if target.is_file() else target.rglob("*.py")
        for file_path in candidates:
            content = file_path.read_text(encoding="utf-8")
            allowed_names = (
                {"_make_downloader"}
                if file_path.name == "music_service.py"
                else set()
            )
            for export in sorted(
                find_python_effect_exports(content, allowed_names=allowed_names)
            ):
                violations.append(
                    f"{file_path.relative_to(PROJECT_ROOT)}: 禁止的 effect `{export}`"
                )

    if violations:
        print("✗ Python facade/service 仍暴露非音乐 execution effect")
        for violation in violations:
            print(f"  - {violation}")
        return False

    print("✓ Python facade/service 不含非音乐 execution effect")
    return True


def check_music_allowlist():
    """检查 py_download_music 及其最小调用链完整"""
    handler_path = PROJECT_ROOT / "src-tauri" / "src" / "python" / "handler.rs"
    mod_path = PROJECT_ROOT / "src-tauri" / "src" / "python" / "mod.rs"
    commands_path = PROJECT_ROOT / "src-tauri" / "src" / "commands" / "python.rs"
    lib_path = PROJECT_ROOT / "src-tauri" / "src" / "lib.rs"

    violations = []

    # 1. handler.rs 必须有 download_music
    if handler_path.exists():
        content = handler_path.read_text(encoding="utf-8")
        if "pub fn download_music(" not in content:
            violations.append("handler.rs 缺少 `pub fn download_music`")
        if "call_py_json(\"download_music\"" not in content:
            violations.append("handler.rs 的 `download_music` 未调用 Python `download_music`")

    # 2. mod.rs 必须有 download_music re-export
    if mod_path.exists():
        content = mod_path.read_text(encoding="utf-8")
        if "download_music" not in content:
            violations.append("mod.rs 缺少 `download_music` re-export")

    # 3. commands/python.rs 必须有 py_download_music
    if commands_path.exists():
        content = commands_path.read_text(encoding="utf-8")
        if "py_download_music" not in content:
            violations.append("commands/python.rs 缺少 `py_download_music` command")

    # 4. lib.rs 必须有 py_download_music registration
    if lib_path.exists():
        content = lib_path.read_text(encoding="utf-8")
        if "py_download_music" not in content:
            violations.append("lib.rs 缺少 `py_download_music` command registration")

    # 5. py_bridge.py 必须有 download_music
    py_bridge_path = PROJECT_ROOT / "core" / "bridge" / "py_bridge.py"
    if py_bridge_path.exists():
        content = py_bridge_path.read_text(encoding="utf-8")
        if "def download_music(" not in content:
            violations.append("py_bridge.py 缺少 `def download_music`")
        if "def download_music_batch(" not in content:
            violations.append("py_bridge.py 缺少 `def download_music_batch`")

    # 6. __init__.py 必须有 download_music in py_bridge imports
    core_init = PROJECT_ROOT / "core" / "__init__.py"
    if core_init.exists():
        content = core_init.read_text(encoding="utf-8")
        if "download_music" not in content:
            violations.append("core/__init__.py 缺少 `download_music` export")
        if "download_music_batch" not in content:
            violations.append("core/__init__.py 缺少 `download_music_batch` export")

    # 7. Frontend must have py_download_music
    search_dir = PROJECT_ROOT / "src"
    found_frontend = False
    for file_path in search_dir.rglob("*"):
        if file_path.suffix not in ('.ts', '.tsx'):
            continue
        try:
            content = file_path.read_text(encoding="utf-8")
            if 'py_download_music' in content:
                found_frontend = True
                break
        except (UnicodeDecodeError, PermissionError):
            continue
    if not found_frontend:
        violations.append("前端缺少 `py_download_music` 引用")

    if violations:
        print("✗ music allowlist 检查失败：")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✓ Music allowlist 完整：py_download_music 及其最小调用链就位")
    return True


def check_task_service_path():
    """Scan the real task_service.rs path for legacy Python execution calls."""
    task_service_path = PROJECT_ROOT / "src-tauri" / "src" / "services" / "download" / "task_service.rs"
    if not task_service_path.exists():
        print(f"✗ task_service.rs 不存在于真实路径: {task_service_path}")
        return False

    content = task_service_path.read_text(encoding="utf-8")
    forbidden = {
        "crate::python::handler::resolve_urls",
        "crate::python::handler::start_download",
        "resolve_page_filtered",
    }
    violations = sorted(pattern for pattern in forbidden if pattern in content)
    required = {
        "crate::python::handler::resolve_single",
        "crate::python::handler::resolve_music_urls",
        "crate::python::handler::resolve_live",
        "resolve_paged_download_plan",
    }
    missing = sorted(pattern for pattern in required if pattern not in content)
    if violations or missing:
        print("✗ task_service.rs 未保持 typed resolver 边界")
        for pattern in violations:
            print(f"  - 仍包含 legacy 调用: {pattern}")
        for pattern in missing:
            print(f"  - 缺少 typed resolver 路径: {pattern}")
        return False

    print(f"✓ 扫描真实路径: {task_service_path.relative_to(PROJECT_ROOT)}")
    return True


def check_negation_fixture():
    """Run deliberately-invalid source through the production matchers."""
    tests = [
        (
            "新增第二个 Python download command",
            find_python_execution_commands,
            "generate_handler![py_download_extra]",
        ),
        (
            "Rust Python handler execution export",
            find_python_effect_exports,
            "pub fn download_video(url: &str) {}",
        ),
        (
            "Python facade execution export",
            find_python_effect_exports,
            "def start_download(mode, url): pass",
        ),
        (
            "Python service execution method",
            find_python_effect_exports,
            "async def handle_one_video(self, url): pass",
        ),
        (
            "Python task DB write",
            find_forbidden_db_refs,
            'db.execute("INSERT INTO download_tasks ...")',
        ),
        (
            "Python lifecycle event",
            find_forbidden_event_calls,
            'emit("task-update", payload)',
        ),
    ]

    failed = []
    for name, matcher, source in tests:
        if matcher(source):
            print(f"    [自测通过] {name}")
        else:
            failed.append(name)
            print(f"    [自测失败] {name}")

    safe_music = "pub async fn py_download_music() {}\ndef download_music(url): pass"
    if find_python_execution_commands(safe_music) or find_python_effect_exports(safe_music):
        failed.append("音乐 allowlist")
        print("    [自测失败] 音乐 allowlist")
    else:
        print("    [自测通过] 音乐 allowlist")

    if failed:
        print(f"✗ [内部自测失败] 未正确匹配: {', '.join(failed)}")
        return False

    print(f"✓ [内部自测] {len(tests)} 个违规与音乐例外均通过真实 matcher")
    return True


# ============================================================
# 主入口
# ============================================================

def main():
    print("=" * 60)
    print("架构边界检查 (Issue 10 强化版)")
    print("=" * 60)

    checks = [
        ("Tauri command registry 不含非音乐 Python execution", check_no_legacy_python_execution_in_command_registry),
        ("Rust handler.rs 不含旧 execution wrapper", check_no_legacy_python_execution_in_handler),
        ("Rust mod.rs re-export 不含旧 execution", check_no_legacy_execution_in_mod_reexport),
        ("Python facade/service 不含非音乐 execution", check_no_python_effect_exports),
        ("Python 业务层不写禁止的 DB 表", check_no_python_db_writes),
        ("Python 业务层不发非音乐 lifecycle event", check_no_python_non_music_events),
        ("前端仅调用 Rust-owned commands", check_frontend_uses_rust_commands),
        ("Music allowlist 完整", check_music_allowlist),
        ("扫描真实 task_service.rs 路径", check_task_service_path),
        ("[内部自测] 否定-fixture", check_negation_fixture),
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

    return 0 if all_passed else 1


if __name__ == "__main__":
    sys.exit(main())
