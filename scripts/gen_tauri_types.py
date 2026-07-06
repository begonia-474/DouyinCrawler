"""从 Rust db.rs 生成 TypeScript 类型定义

解析 db.rs 中的 struct 定义，生成 src/lib/tauri-types.ts。
确保 Rust 和 TypeScript 的类型定义始终保持同步。

类型来源策略：
  - 与 bindings.ts（specta 生成）一致的类型 → re-export
  - VideoInfo（62 字段完整 DB 模型）→ 本地定义

用法：python scripts/gen_tauri_types.py
"""

import re
import sys
from pathlib import Path

# 项目根目录
ROOT = Path(__file__).resolve().parent.parent
DB_RS = ROOT / "src-tauri" / "src" / "database" / "models.rs"
OUT_TS = ROOT / "src" / "lib" / "tauri-types.ts"

# Rust → TypeScript 类型映射
TYPE_MAP = {
    "String": "string",
    "Option<String>": "string | null",
    "Option<i64>": "number | null",
    "Option<i32>": "number | null",
    "i32": "number",
    "i64": "number",
    "u32": "number",
    "u64": "number",
    "f64": "number",
    "bool": "boolean",
}

# Vec<T> → T[] 映射（运行时解析）
VEC_PATTERN = re.compile(r"Vec<(\w+)>")

# re-export 自 bindings.ts 的类型（specta 编译期生成，与 db.rs 定义一致）
REEXPORT_TYPES = [
    "UserStats",
    "VideoTypeStat",
    "UserInfo",
    "VideoStats",
    "MusicCollection",
    "LiveRecord",
    "TrendPoint",
    "AuthorStat",
    "StorageStat",
    "DbHealth",
]

# 本地定义的类型（与 bindings.ts 不同，需保留完整版本）
LOCAL_TYPES = [
    "VideoInfo",
]


def parse_structs(content: str) -> dict[str, list[tuple[str, str]]]:
    """解析 db.rs 中的 struct 定义，返回 {struct_name: [(field_name, rust_type), ...]}"""
    structs = {}

    # 匹配 pub struct Name { ... }
    struct_pattern = re.compile(
        r"#\[derive\(.*?\)\]\s*pub struct (\w+)\s*\{([^}]+)\}",
        re.DOTALL,
    )

    all_types = set(REEXPORT_TYPES) | set(LOCAL_TYPES)
    for match in struct_pattern.finditer(content):
        name = match.group(1)
        if name not in all_types:
            continue

        body = match.group(2)
        fields = []

        # 匹配 pub field: Type（忽略 serde 属性行）
        field_pattern = re.compile(
            r"(?:#\[.*?\]\s*)*pub\s+(\w+)\s*:\s*([^,\n]+)"
        )
        for fm in field_pattern.finditer(body):
            field_name = fm.group(1)
            rust_type = fm.group(2).strip()
            fields.append((field_name, rust_type))

        structs[name] = fields

    return structs


def rust_type_to_ts(rust_type: str) -> str:
    """Rust 类型转 TypeScript 类型"""
    t = rust_type.strip()

    # 先查精确匹配
    if t in TYPE_MAP:
        return TYPE_MAP[t]

    # Vec<T> → T[]
    vec_match = VEC_PATTERN.match(t)
    if vec_match:
        inner = vec_match.group(1)
        return f"{rust_type_to_ts(inner)}[]"

    # 已知 struct 名 → 直接使用（如 TypeStat、VideoTypeStat）
    all_types = set(REEXPORT_TYPES) | set(LOCAL_TYPES)
    if t in all_types:
        return t

    return "unknown"


def generate_ts(structs: dict[str, list[tuple[str, str]]]) -> str:
    """生成 TypeScript 代码（re-export + 本地定义）"""
    lines = [
        "// ============================================================",
        "// 此文件由 scripts/gen_tauri_types.py 自动生成",
        "// 源头：src-tauri/src/db.rs + src-tauri/src/database/models.rs",
        "//",
        "// 类型来源：",
        "//   - 12 个类型 re-export 自 bindings.ts（specta 编译期生成，权威来源）",
        "//   - VideoInfo 本地定义（62 字段完整 DB 模型，bindings.ts 仅 18 字段）",
        "//",
        "// 修改后请运行: python scripts/gen_tauri_types.py",
        "// ============================================================",
        "",
        "// Re-export 与 bindings.ts 一致的类型（单一来源）",
    ]

    # 生成 re-export
    reexport_names = [name for name in REEXPORT_TYPES if name in structs]
    if reexport_names:
        lines.append("export type {")
        for name in reexport_names:
            lines.append(f"  {name},")
        lines.append('} from "./bindings";')
    lines.append("")

    # 生成本地定义的类型
    for struct_name in LOCAL_TYPES:
        if struct_name not in structs:
            continue

        fields = structs[struct_name]
        lines.append(f"// {struct_name}：完整 DB 模型（{len(fields)} 字段）")
        lines.append(f"// bindings.ts 仅有 18 个核心字段（用于 task service），此处保留完整版本供 library 页面和 DB 查询使用")
        lines.append(f"export interface {struct_name} {{")

        for field_name, rust_type in fields:
            ts_type = rust_type_to_ts(rust_type)
            lines.append(f"  {field_name}: {ts_type};")

        lines.append("}")
        lines.append("")

    return "\n".join(lines)


def main():
    if not DB_RS.exists():
        print(f"错误：找不到 {DB_RS}")
        sys.exit(1)

    content = DB_RS.read_text(encoding="utf-8")
    structs = parse_structs(content)

    if not structs:
        print("错误：未解析到任何 struct")
        sys.exit(1)

    ts_code = generate_ts(structs)
    OUT_TS.write_text(ts_code, encoding="utf-8")

    print(f"已生成 {OUT_TS}")
    for name, fields in structs.items():
        label = "re-export" if name in REEXPORT_TYPES else "本地定义"
        print(f"  {name}: {len(fields)} 个字段 ({label})")


if __name__ == "__main__":
    main()
