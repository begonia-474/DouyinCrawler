//! 查询构建工具 — Upsert SQL 生成器

/// Upsert 列类型
#[derive(Clone, Copy)]
pub enum ColKind {
    /// 普通字段：冲突时直接覆盖
    Normal,
    /// 时效性字段：冲突时仅当新值非空才覆盖
    Volatile,
    /// 统计字段：冲突时取 MAX（只增不减）
    Stat,
}

/// 构建 upsert SQL：INSERT ... ON CONFLICT(pk) DO UPDATE SET ...
/// - Normal: 冲突时直接覆盖
/// - Volatile: 冲突时仅当新值非空才覆盖（时效性字段）
/// - Stat: 冲突时取 MAX（只增不减）
pub fn build_upsert_sql(table: &str, pk: &str, cols: &[(&str, ColKind)]) -> String {
    let col_names: Vec<&str> = cols.iter().map(|(name, _)| *name).collect();
    let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("?{}", i)).collect();

    let mut set_parts = Vec::new();
    for (_i, (name, kind)) in cols.iter().enumerate() {
        match kind {
            ColKind::Normal => {
                set_parts.push(format!("{} = excluded.{}", name, name));
            }
            ColKind::Volatile => {
                set_parts.push(format!(
                    "{0} = CASE WHEN excluded.{0} IS NOT NULL AND excluded.{0} != '' \
                     THEN excluded.{0} ELSE {1}.{0} END",
                    name, table
                ));
            }
            ColKind::Stat => {
                set_parts.push(format!(
                    "{0} = CASE WHEN excluded.{0} > {1}.{0} OR {1}.{0} IS NULL \
                     THEN excluded.{0} ELSE {1}.{0} END",
                    name, table
                ));
            }
        }
    }

    format!(
        "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT({}) DO UPDATE SET {}",
        table,
        col_names.join(", "),
        placeholders.join(", "),
        pk,
        set_parts.join(", ")
    )
}
