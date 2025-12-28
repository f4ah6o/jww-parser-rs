//! JWW (Jw_cad) ファイルパーサー
//!
//! Jw_cadで使用されるJWWバイナリファイル形式をパースし、
//! Rustデータ構造に変換するライブラリ。

mod error;
mod reader;
mod types;

pub use error::{ParseError, Result};
pub use reader::Reader;
pub use types::{
    Document, Entity, EntityBase, Layer, LayerGroup,
    Line, Arc, Point, Text, Solid, Block, BlockDef,
};

/// JWWファイルをパースする
///
/// # 引数
/// * `data` - JWWファイルのバイナリデータ
///
/// # 戻り値
/// パースされたドキュメント
///
/// # エラー
/// - 無効なシグネチャ
/// - 不正なファイル構造
/// - IOエラー
pub fn parse(data: &[u8]) -> Result<Document> {
    // シグネチャ検証
    if data.len() < 8 || &data[0..8] != b"JwwData." {
        return Err(ParseError::InvalidSignature);
    }

    let mut reader = Reader::new(&data[8..]);

    // バージョン読み取り
    let version = reader.read_dword()?;

    // ヘッダー情報読み取り
    let memo = reader.read_cstring()?;
    let paper_size = reader.read_dword()?;
    let write_layer_group = reader.read_dword()?;

    // レイヤグループ読み取り (16グループ)
    let mut layer_groups: [LayerGroup; 16] = std::array::from_fn(|_| LayerGroup::default());
    for g_lay in 0..16 {
        let state = reader.read_dword()?;
        let write_layer = reader.read_dword()?;
        let scale = reader.read_double()?;
        let protect = reader.read_dword()?;

        let mut layers: [Layer; 16] = std::array::from_fn(|_| Layer::default());
        for lay in 0..16 {
            let lay_state = reader.read_dword()?;
            let lay_protect = reader.read_dword()?;
            layers[lay as usize] = Layer {
                state: lay_state,
                protect: lay_protect,
                name: String::new(),
            };
        }

        layer_groups[g_lay as usize] = LayerGroup {
            state,
            write_layer,
            scale,
            protect,
            layers,
            name: String::new(),
        };
    }

    // エンティティリスト開始位置を探索
    let entity_list_offset = find_entity_list_offset(data, version);
    let entity_list_offset = match entity_list_offset {
        Some(offset) => offset,
        None => return Err(ParseError::Other("could not find entity list".to_string())),
    };

    // エンティティをパース
    let entity_data = &data[entity_list_offset..];
    let mut reader2 = Reader::new(entity_data);
    let entities = parse_entity_list(&mut reader2, version)?;

    // TODO: ブロック定義のパース

    // レイヤー名の設定（デフォルト名を使用）
    for g_lay in 0..16 {
        if layer_groups[g_lay as usize].name.is_empty() {
            layer_groups[g_lay as usize].name = format!("Group{:X}", g_lay);
        }
        for lay in 0..16 {
            if layer_groups[g_lay as usize].layers[lay as usize].name.is_empty() {
                layer_groups[g_lay as usize].layers[lay as usize].name = format!("{:X}-{:X}", g_lay, lay);
            }
        }
    }

    Ok(Document {
        version,
        memo,
        paper_size,
        write_layer_group,
        layer_groups,
        entities,
        block_defs: Vec::new(),
    })
}

/// エンティティリストの開始位置を探索する
fn find_entity_list_offset(data: &[u8], version: u32) -> Option<usize> {
    let schema_bytes = [version as u8, (version >> 8) as u8];

    for i in 100..data.len().saturating_sub(20) {
        // 0xFF 0xFF (新しいクラス定義マーカー) を検索
        if data[i] == 0xFF && data[i + 1] == 0xFF {
            // スキーマバージョンが一致するか確認
            if data[i + 2] == schema_bytes[0] && data[i + 3] == schema_bytes[1] {
                // クラス名長さを取得
                let name_len = (data[i + 4] as u16) | ((data[i + 5] as u16) << 8);
                if name_len >= 8 && name_len <= 20 && i + 6 + name_len as usize <= data.len() {
                    let class_name = &data[i + 6..i + 6 + name_len as usize];
                    if class_name.starts_with(b"CData") {
                        // 最初のエンティティクラス定義が見つかった
                        // カウントWORDは直前（2バイト前）
                        return Some(i.saturating_sub(2));
                    }
                }
            }
        }
    }

    None
}

/// エンティティリストをパースする
fn parse_entity_list<R: std::io::Read>(reader: &mut Reader<R>, version: u32) -> Result<Vec<Entity>> {
    let count = reader.read_word()? as u32;

    let mut entities = Vec::with_capacity(count as usize);

    // MFC CArchive PIDトラッキング
    let mut pid_to_class: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
    let mut next_pid: u32 = 1;

    for _ in 0..count {
        match parse_entity_with_pid_tracking(reader, version, &mut pid_to_class, &mut next_pid) {
            Ok(Some(entity)) => entities.push(entity),
            Ok(None) => {} // Nullオブジェクトはスキップ
            Err(e) => return Err(e),
        }
    }

    Ok(entities)
}

/// PIDトラッキング付きでエンティティをパースする
fn parse_entity_with_pid_tracking<R: std::io::Read>(
    reader: &mut Reader<R>,
    version: u32,
    pid_to_class: &mut std::collections::HashMap<u32, String>,
    next_pid: &mut u32,
) -> Result<Option<Entity>> {
    let class_id = reader.read_word()?;

    let class_name = match class_id {
        0xFFFF => {
            // 新しいクラス定義
            let _schema = reader.read_word()?;
            let name_len = reader.read_word()?;
            let mut name_buf = vec![0u8; name_len as usize];
            reader.read_exact(&mut name_buf)?;
            let class_name = String::from_utf8_lossy(&name_buf).to_string();

            pid_to_class.insert(*next_pid, class_name.clone());
            *next_pid += 1;
            class_name
        }
        0x8000 => {
            // Nullオブジェクト
            return Ok(None);
        }
        _ => {
            // クラス参照: 0x8000 | class_pid
            let class_pid = (class_id & 0x7FFF) as u32;
            pid_to_class
                .get(&class_pid)
                .cloned()
                .ok_or(ParseError::UnknownClassPid(class_pid))?
        }
    };

    // クラス名に応じてエンティティをパース
    let entity = match class_name.as_str() {
        "CDataSen" => {
            let base = parse_entity_base(reader, version)?;
            let start_x = reader.read_double()?;
            let start_y = reader.read_double()?;
            let end_x = reader.read_double()?;
            let end_y = reader.read_double()?;
            Some(Entity::Line(Line {
                base,
                start_x,
                start_y,
                end_x,
                end_y,
            }))
        }
        "CDataEnko" => {
            let base = parse_entity_base(reader, version)?;
            let center_x = reader.read_double()?;
            let center_y = reader.read_double()?;
            let radius = reader.read_double()?;
            let start_angle = reader.read_double()?;
            let arc_angle = reader.read_double()?;
            let tilt_angle = reader.read_double()?;
            let flatness = reader.read_double()?;
            let full_circle = reader.read_dword()?;
            Some(Entity::Arc(Arc {
                base,
                center_x,
                center_y,
                radius,
                start_angle,
                arc_angle,
                tilt_angle,
                flatness,
                is_full_circle: full_circle != 0,
            }))
        }
        "CDataTen" => {
            let base = parse_entity_base(reader, version)?;
            let x = reader.read_double()?;
            let y = reader.read_double()?;
            let tmp = reader.read_dword()?;
            let is_temporary = tmp != 0;

            let mut code = 0;
            let mut angle = 0.0;
            let mut scale = 1.0;
            if base.pen_style == 100 {
                code = reader.read_dword()?;
                angle = reader.read_double()?;
                scale = reader.read_double()?;
            }
            Some(Entity::Point(Point {
                base,
                x,
                y,
                is_temporary,
                code,
                angle,
                scale,
            }))
        }
        "CDataMoji" => {
            let base = parse_entity_base(reader, version)?;
            let start_x = reader.read_double()?;
            let start_y = reader.read_double()?;
            let end_x = reader.read_double()?;
            let end_y = reader.read_double()?;
            let text_type = reader.read_dword()?;
            let size_x = reader.read_double()?;
            let size_y = reader.read_double()?;
            let spacing = reader.read_double()?;
            let angle = reader.read_double()?;
            let font_name = reader.read_cstring()?;
            let content = reader.read_cstring()?;
            Some(Entity::Text(Text {
                base,
                start_x,
                start_y,
                end_x,
                end_y,
                text_type,
                size_x,
                size_y,
                spacing,
                angle,
                font_name,
                content,
            }))
        }
        "CDataSolid" => {
            let base = parse_entity_base(reader, version)?;
            let point1_x = reader.read_double()?;
            let point1_y = reader.read_double()?;
            let point4_x = reader.read_double()?;
            let point4_y = reader.read_double()?;
            let point2_x = reader.read_double()?;
            let point2_y = reader.read_double()?;
            let point3_x = reader.read_double()?;
            let point3_y = reader.read_double()?;

            let mut color = 0;
            if base.pen_color == 10 {
                color = reader.read_dword()?;
            }
            Some(Entity::Solid(Solid {
                base,
                point1_x,
                point1_y,
                point2_x,
                point2_y,
                point3_x,
                point3_y,
                point4_x,
                point4_y,
                color,
            }))
        }
        "CDataBlock" => {
            let base = parse_entity_base(reader, version)?;
            let ref_x = reader.read_double()?;
            let ref_y = reader.read_double()?;
            let scale_x = reader.read_double()?;
            let scale_y = reader.read_double()?;
            let rotation = reader.read_double()?;
            let def_number = reader.read_dword()?;
            Some(Entity::Block(Block {
                base,
                ref_x,
                ref_y,
                scale_x,
                scale_y,
                rotation,
                def_number,
            }))
        }
        "CDataSunpou" => {
            // 寸法エンティティ - 簡易的に線として扱う
            let _base = parse_entity_base(reader, version)?;
            // 線メンバーをパース
            let _line_base = parse_entity_base(reader, version)?;
            let _start_x = reader.read_double()?;
            let _start_y = reader.read_double()?;
            let _end_x = reader.read_double()?;
            let _end_y = reader.read_double()?;
            // 文字メンバーをパース（スキップ）
            let _text_base = parse_entity_base(reader, version)?;
            let _text_start_x = reader.read_double()?;
            let _text_start_y = reader.read_double()?;
            let _text_end_x = reader.read_double()?;
            let _text_end_y = reader.read_double()?;
            let _text_type = reader.read_dword()?;
            let _text_size_x = reader.read_double()?;
            let _text_size_y = reader.read_double()?;
            let _text_spacing = reader.read_double()?;
            let _text_angle = reader.read_double()?;
            let _text_font_name = reader.read_cstring()?;
            let _text_content = reader.read_cstring()?;

            // Ver 4.20+ の追加データ
            if version >= 420 {
                let _sxf_mode = reader.read_word()?;
                for _ in 0..2 {
                    let _ = parse_entity_base(reader, version)?;
                    let _ = reader.read_double()?;
                    let _ = reader.read_double()?;
                    let _ = reader.read_double()?;
                    let _ = reader.read_double()?;
                }
                for _ in 0..4 {
                    let _ = parse_entity_base(reader, version)?;
                    let _ = reader.read_double()?;
                    let _ = reader.read_double()?;
                    let _ = reader.read_dword()?;
                }
            }
            // 寸法はスキップ
            None
        }
        _ => return Err(ParseError::UnknownEntityClass(class_name)),
    };

    *next_pid += 1;
    Ok(entity)
}

/// エンティティ基本属性をパースする
fn parse_entity_base<R: std::io::Read>(reader: &mut Reader<R>, version: u32) -> Result<EntityBase> {
    let group = reader.read_dword()?;
    let pen_style = reader.read_byte()?;
    let pen_color = reader.read_word()?;

    let pen_width = if version >= 351 {
        reader.read_word()?
    } else {
        0
    };

    let layer = reader.read_word()?;
    let layer_group = reader.read_word()?;
    let flag = reader.read_word()?;

    Ok(EntityBase {
        group,
        pen_style,
        pen_color,
        pen_width,
        layer,
        layer_group,
        flag,
    })
}
