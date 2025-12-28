//! DXFファイルを解析してエンティティ情報を抽出するモジュール
//!
//! Go版とRust版のDXF出力を比較するための簡易DXFパーサー

use std::collections::HashMap;

/// DXFから抽出したエンティティ情報
#[derive(Debug, Clone, PartialEq)]
pub struct DxfEntity {
    /// エンティティタイプ (LINE, CIRCLE, ARC, ELLIPSE, POINT, TEXT, SOLID, INSERT)
    pub entity_type: DxfEntityType,
    /// レイヤー名
    pub layer: String,
    /// 色番号
    pub color: i32,
    /// 線種名
    pub line_type: String,
    /// 座標と属性値 (group_code -> value)
    pub properties: HashMap<i32, String>,
}

/// DXFエンティティタイプ
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DxfEntityType {
    Line,
    Circle,
    Arc,
    Ellipse,
    Point,
    Text,
    Solid,
    Insert,
    Unknown(String),
}

impl DxfEntityType {
    fn from_str(s: &str) -> Self {
        match s {
            "LINE" => DxfEntityType::Line,
            "CIRCLE" => DxfEntityType::Circle,
            "ARC" => DxfEntityType::Arc,
            "ELLIPSE" => DxfEntityType::Ellipse,
            "POINT" => DxfEntityType::Point,
            "TEXT" => DxfEntityType::Text,
            "SOLID" => DxfEntityType::Solid,
            "INSERT" => DxfEntityType::Insert,
            other => DxfEntityType::Unknown(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DxfEntityType::Line => "LINE",
            DxfEntityType::Circle => "CIRCLE",
            DxfEntityType::Arc => "ARC",
            DxfEntityType::Ellipse => "ELLIPSE",
            DxfEntityType::Point => "POINT",
            DxfEntityType::Text => "TEXT",
            DxfEntityType::Solid => "SOLID",
            DxfEntityType::Insert => "INSERT",
            DxfEntityType::Unknown(s) => s,
        }
    }
}

/// DXF文字列を解析してエンティティリストとレイヤーリストを返す
pub fn parse_dxf_entities(dxf_content: &str) -> (Vec<DxfEntity>, Vec<String>) {
    let mut entities = Vec::new();
    let mut layers: Vec<String> = Vec::new();
    let lines: Vec<&str> = dxf_content.lines().collect();

    let mut i = 0;
    let mut in_entities_section = false;
    let mut in_tables_section = false;
    let mut in_layer_table = false;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            i += 1;
            continue;
        }

        // セクションの検出
        if line == "SECTION" {
            if i + 2 < lines.len() && lines[i + 1].trim() == "2" {
                let section_name = lines[i + 2].trim();
                match section_name {
                    "ENTITIES" => in_entities_section = true,
                    "TABLES" => in_tables_section = true,
                    "BLOCKS" => in_entities_section = false,
                    _ => {}
                }
            }
            i += 3;
            continue;
        }

        if line == "ENDSEC" {
            in_entities_section = false;
            in_tables_section = false;
            in_layer_table = false;
            i += 1;
            continue;
        }

        // LAYERテーブルの検出
        if in_tables_section && line == "TABLE" {
            if i + 2 < lines.len() && lines[i + 1].trim() == "2" {
                let table_name = lines[i + 2].trim();
                if table_name == "LAYER" {
                    in_layer_table = true;
                }
            }
            i += 3;
            continue;
        }

        if line == "ENDTAB" {
            in_layer_table = false;
            i += 1;
            continue;
        }

        // LAYERデータの収集（"  0"の次が"LAYER"の場合のみ）
        if in_layer_table && line == "0" {
            if i + 1 < lines.len() && lines[i + 1].trim() == "LAYER" {
                if let Some(layer_name) = find_next_group_value(&lines, i + 1, "2") {
                    layers.push(layer_name);
                }
            }
        }

        // エンティティの解析
        if in_entities_section {
            if let Some(entity_type) = parse_entity_at(&lines, i) {
                entities.push(entity_type);
            }
        }

        i += 1;
    }

    (entities, layers)
}

/// 指定された位置からエンティティを解析する
fn parse_entity_at(lines: &[&str], start: usize) -> Option<DxfEntity> {
    let line = lines[start].trim();

    let entity_type_str = match line {
        "LINE" | "CIRCLE" | "ARC" | "ELLIPSE" | "POINT" | "TEXT" | "SOLID" | "INSERT" => line,
        _ => return None,
    };

    let entity_type = DxfEntityType::from_str(entity_type_str);

    // エンティティの終わり（次の0またはENDSEC）まで読む
    let mut layer = String::from("0");
    let mut color = 7; // デフォルト色
    let mut line_type = String::from("CONTINUOUS");
    let mut properties = HashMap::new();

    let mut i = start + 1;
    while i < lines.len() {
        let current = lines[i].trim();
        if current == "0" {
            break; // 次のエンティティ
        }

        // グループコードと値のペアを処理
        if let Ok(group_code) = current.parse::<i32>() {
            if i + 1 < lines.len() {
                let value = lines[i + 1].trim();

                // 重要なプロパティを保存
                match group_code {
                    8 => layer = value.to_string(),  // レイヤー
                    62 => {
                        if let Ok(c) = value.parse::<i32>() {
                            color = c;
                        }
                    }  // 色
                    6 => line_type = value.to_string(),  // 線種
                    _ => {
                        properties.insert(group_code, value.to_string());
                    }
                }
            }
        }

        i += 2;
    }

    Some(DxfEntity {
        entity_type,
        layer,
        color,
        line_type,
        properties,
    })
}

/// 指定されたグループコードの値を次の行から見つける
fn find_next_group_value(lines: &[&str], start: usize, group_code: &str) -> Option<String> {
    for i in start..lines.len().saturating_sub(1) {
        if lines[i].trim() == group_code {
            return Some(lines[i + 1].trim().to_string());
        }
    }
    None
}

/// 2つのエンティティリストを比較する
pub fn compare_dxf_entities(
    go_entities: &[DxfEntity],
    rust_entities: &[DxfEntity],
    tolerance: f64,
) -> Vec<EntityDifference> {
    let mut differences = Vec::new();

    // エンティティ数の比較
    if go_entities.len() != rust_entities.len() {
        differences.push(EntityDifference::EntityCountMismatch {
            go: go_entities.len(),
            rust: rust_entities.len(),
        });
    }

    let min_len = go_entities.len().min(rust_entities.len());

    for i in 0..min_len {
        let go_ent = &go_entities[i];
        let rust_ent = &rust_entities[i];

        // エンティティタイプの比較
        if go_ent.entity_type != rust_ent.entity_type {
            differences.push(EntityDifference::TypeMismatch {
                index: i,
                go_type: go_ent.entity_type.as_str().to_string(),
                rust_type: rust_ent.entity_type.as_str().to_string(),
            });
        }

        // レイヤーの比較
        if go_ent.layer != rust_ent.layer {
            differences.push(EntityDifference::LayerMismatch {
                index: i,
                go_layer: go_ent.layer.clone(),
                rust_layer: rust_ent.layer.clone(),
            });
        }

        // 色の比較
        if go_ent.color != rust_ent.color {
            differences.push(EntityDifference::ColorMismatch {
                index: i,
                go_color: go_ent.color,
                rust_color: rust_ent.color,
            });
        }

        // 座標値の比較
        compare_coordinates(&go_ent.properties, &rust_ent.properties, i, tolerance, &mut differences);
    }

    differences
}

/// 座標プロパティを比較する
fn compare_coordinates(
    go_props: &HashMap<i32, String>,
    rust_props: &HashMap<i32, String>,
    index: usize,
    tolerance: f64,
    differences: &mut Vec<EntityDifference>,
) {
    // 座標グループコード
    let coord_codes = [10, 11, 12, 13, 20, 21, 22, 23, 40, 41, 42, 50, 51];

    for code in coord_codes {
        let go_val = go_props.get(&code);
        let rust_val = rust_props.get(&code);

        match (go_val, rust_val) {
            (Some(go_str), Some(rust_str)) => {
                if let (Ok(go_f), Ok(rust_f)) = (go_str.parse::<f64>(), rust_str.parse::<f64>()) {
                    if (go_f - rust_f).abs() > tolerance {
                        differences.push(EntityDifference::CoordinateMismatch {
                            index,
                            group_code: code,
                            go_value: go_f,
                            rust_value: rust_f,
                        });
                    }
                }
            }
            (Some(_), None) | (None, Some(_)) => {
                differences.push(EntityDifference::MissingCoordinate {
                    index,
                    group_code: code,
                });
            }
            (None, None) => {}
        }
    }
}

/// エンティティの差異
#[derive(Debug)]
pub enum EntityDifference {
    EntityCountMismatch { go: usize, rust: usize },
    TypeMismatch {
        index: usize,
        go_type: String,
        rust_type: String,
    },
    LayerMismatch {
        index: usize,
        go_layer: String,
        rust_layer: String,
    },
    ColorMismatch {
        index: usize,
        go_color: i32,
        rust_color: i32,
    },
    CoordinateMismatch {
        index: usize,
        group_code: i32,
        go_value: f64,
        rust_value: f64,
    },
    MissingCoordinate {
        index: usize,
        group_code: i32,
    },
}

impl std::fmt::Display for EntityDifference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityDifference::EntityCountMismatch { go, rust } => {
                write!(f, "エンティティ数不一致: Go={}, Rust={}", go, rust)
            }
            EntityDifference::TypeMismatch { index, go_type, rust_type } => {
                write!(f, "エンティティ[{}] タイプ不一致: Go={}, Rust={}", index, go_type, rust_type)
            }
            EntityDifference::LayerMismatch { index, go_layer, rust_layer } => {
                write!(f, "エンティティ[{}] レイヤー不一致: Go={}, Rust={}", index, go_layer, rust_layer)
            }
            EntityDifference::ColorMismatch { index, go_color, rust_color } => {
                write!(f, "エンティティ[{}] 色不一致: Go={}, Rust={}", index, go_color, rust_color)
            }
            EntityDifference::CoordinateMismatch { index, group_code, go_value, rust_value } => {
                write!(
                    f,
                    "エンティティ[{}] 座標不一致(コード{}): Go={}, Rust={}",
                    index, group_code, go_value, rust_value
                )
            }
            EntityDifference::MissingCoordinate { index, group_code } => {
                write!(f, "エンティティ[{}] 座標コード{}が不足", index, group_code)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_line_entity() {
        let dxf = r#"0
SECTION
2
ENTITIES
0
LINE
8
0
62
5
6
CONTINUOUS
10
0.0
20
0.0
11
100.0
21
50.0
0
ENDSEC
0
EOF"#;

        let (entities, _layers) = parse_dxf_entities(dxf);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].entity_type, DxfEntityType::Line);
        assert_eq!(entities[0].layer, "0");
        assert_eq!(entities[0].color, 5);
        assert_eq!(entities[0].line_type, "CONTINUOUS");
    }

    #[test]
    fn test_parse_circle_entity() {
        let dxf = r#"0
SECTION
2
ENTITIES
0
CIRCLE
8
0
62
1
10
50.0
20
50.0
40
25.0
0
ENDSEC
0
EOF"#;

        let (entities, _layers) = parse_dxf_entities(dxf);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].entity_type, DxfEntityType::Circle);
    }

    #[test]
    fn test_compare_identical_entities() {
        let dxf = r#"0
SECTION
2
ENTITIES
0
LINE
8
0
62
5
10
0.0
20
0.0
11
100.0
21
50.0
0
ENDSEC
0
EOF"#;

        let (entities, _) = parse_dxf_entities(dxf);
        let differences = compare_dxf_entities(&entities, &entities, 0.001);
        assert!(differences.is_empty());
    }

    #[test]
    fn test_parse_layers() {
        let dxf = r#"0
SECTION
2
TABLES
0
TABLE
2
LAYER
70
2
0
LAYER
2
Layer1
62
1
6
CONTINUOUS
0
LAYER
2
Layer2
62
2
6
CONTINUOUS
0
ENDTAB
0
ENDSEC
0
EOF"#;

        let (_entities, layers) = parse_dxf_entities(dxf);
        assert_eq!(layers.len(), 2);
        assert!(layers.contains(&"Layer1".to_string()));
        assert!(layers.contains(&"Layer2".to_string()));
    }
}
