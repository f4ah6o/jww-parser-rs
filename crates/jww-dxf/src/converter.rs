//! JWWからDXFへの変換ロジック

use crate::types::*;
use jww_core::{Document as JwwDocument, Entity as JwwEntity};

/// JWWドキュメントをDXFドキュメントに変換する
pub fn convert_document(jww_doc: &JwwDocument) -> Document {
    let layers = convert_layers(jww_doc);
    let entities = convert_entities(jww_doc);
    let blocks = convert_blocks(jww_doc);

    Document {
        layers,
        entities,
        blocks,
    }
}

/// JWWレイヤーをDXFレイヤーに変換する
fn convert_layers(jww_doc: &JwwDocument) -> Vec<Layer> {
    let mut layers = Vec::new();

    for g_lay in 0..16 {
        let lg = &jww_doc.layer_groups[g_lay as usize];
        for lay in 0..16 {
            let l = &lg.layers[lay as usize];
            let name = if l.name.is_empty() {
                format!("{:X}-{:X}", g_lay, lay)
            } else {
                l.name.clone()
            };

            layers.push(Layer {
                name,
                color: ((g_lay * 16 + lay) % 255 + 1) as i32,
                line_type: "CONTINUOUS".to_string(),
                frozen: l.state == 0,
                locked: l.protect != 0,
            });
        }
    }

    layers
}

/// JWWエンティティをDXFエンティティに変換する
fn convert_entities(jww_doc: &JwwDocument) -> Vec<Entity> {
    let mut entities = Vec::new();

    for jww_entity in &jww_doc.entities {
        if let Some(dxf_entity) = convert_entity(jww_entity, jww_doc) {
            entities.push(dxf_entity);
        }
    }

    entities
}

/// 単一のJWWエンティティをDXFエンティティに変換する
fn convert_entity(jww_entity: &JwwEntity, jww_doc: &JwwDocument) -> Option<Entity> {
    let base = jww_entity.base();
    let layer_name = get_layer_name(jww_doc, base.layer_group, base.layer);
    let color = map_color(base.pen_color);
    let line_type = map_line_type(base.pen_style);

    match jww_entity {
        JwwEntity::Line(line) => Some(Entity::Line(Line {
            layer: layer_name,
            color,
            line_type,
            x1: line.start_x,
            y1: line.start_y,
            x2: line.end_x,
            y2: line.end_y,
        })),

        JwwEntity::Arc(arc) => {
            if arc.is_full_circle && arc.flatness == 1.0 {
                // 完全円
                Some(Entity::Circle(Circle {
                    layer: layer_name,
                    color,
                    line_type,
                    center_x: arc.center_x,
                    center_y: arc.center_y,
                    radius: arc.radius,
                }))
            } else if arc.flatness != 1.0 {
                // 楕円または楕円弧
                let major_radius = arc.radius;
                let minor_ratio = arc.flatness;
                let tilt_angle = arc.tilt_angle;

                if minor_ratio > 1.0 {
                    // 軸を入れ替え
                    let major_radius = arc.radius * arc.flatness;
                    let minor_ratio = 1.0 / arc.flatness;
                    let tilt_angle = arc.tilt_angle + std::f64::consts::PI / 2.0;

                    let major_axis_x = major_radius * tilt_angle.cos();
                    let major_axis_y = major_radius * tilt_angle.sin();

                    let (start_param, end_param) = if arc.is_full_circle {
                        (0.0, 2.0 * std::f64::consts::PI)
                    } else {
                        (arc.start_angle, arc.start_angle + arc.arc_angle)
                    };

                    Some(Entity::Ellipse(Ellipse {
                        layer: layer_name,
                        color,
                        line_type,
                        center_x: arc.center_x,
                        center_y: arc.center_y,
                        major_axis_x,
                        major_axis_y,
                        minor_ratio,
                        start_param,
                        end_param,
                    }))
                } else {
                    let major_axis_x = major_radius * tilt_angle.cos();
                    let major_axis_y = major_radius * tilt_angle.sin();

                    let (start_param, end_param) = if arc.is_full_circle {
                        (0.0, 2.0 * std::f64::consts::PI)
                    } else {
                        (arc.start_angle, arc.start_angle + arc.arc_angle)
                    };

                    Some(Entity::Ellipse(Ellipse {
                        layer: layer_name,
                        color,
                        line_type,
                        center_x: arc.center_x,
                        center_y: arc.center_y,
                        major_axis_x,
                        major_axis_y,
                        minor_ratio,
                        start_param,
                        end_param,
                    }))
                }
            } else {
                // 円弧
                let start_angle = rad_to_deg(arc.start_angle);
                let end_angle = rad_to_deg(arc.start_angle + arc.arc_angle);

                Some(Entity::Arc(Arc {
                    layer: layer_name,
                    color,
                    line_type,
                    center_x: arc.center_x,
                    center_y: arc.center_y,
                    radius: arc.radius,
                    start_angle,
                    end_angle,
                }))
            }
        }

        JwwEntity::Point(point) => {
            if point.is_temporary {
                return None; // 仮点はスキップ
            }
            Some(Entity::Point(Point {
                layer: layer_name,
                color,
                line_type,
                x: point.x,
                y: point.y,
            }))
        }

        JwwEntity::Text(text) => {
            let height = if text.size_y <= 0.0 { 2.5 } else { text.size_y };

            Some(Entity::Text(Text {
                layer: layer_name,
                color,
                line_type,
                x: text.start_x,
                y: text.start_y,
                height,
                rotation: text.angle,
                content: text.content.clone(),
                style: "STANDARD".to_string(),
            }))
        }

        JwwEntity::Solid(solid) => Some(Entity::Solid(Solid {
            layer: layer_name,
            color,
            line_type,
            x1: solid.point1_x,
            y1: solid.point1_y,
            x2: solid.point2_x,
            y2: solid.point2_y,
            x3: solid.point3_x,
            y3: solid.point3_y,
            x4: solid.point4_x,
            y4: solid.point4_y,
        })),

        JwwEntity::Block(block) => {
            let block_name = get_block_name(jww_doc, block.def_number);
            Some(Entity::Insert(Insert {
                layer: layer_name,
                color,
                line_type,
                block_name,
                x: block.ref_x,
                y: block.ref_y,
                scale_x: block.scale_x,
                scale_y: block.scale_y,
                rotation: rad_to_deg(block.rotation),
            }))
        }
    }
}

/// JWWブロック定義をDXFブロックに変換する
fn convert_blocks(jww_doc: &JwwDocument) -> Vec<Block> {
    let mut blocks = Vec::new();

    for bd in &jww_doc.block_defs {
        let mut block_entities = Vec::new();

        for e in &bd.entities {
            if let Some(dxf_entity) = convert_entity(e, jww_doc) {
                block_entities.push(dxf_entity);
            }
        }

        blocks.push(Block {
            name: bd.name.clone(),
            base_x: 0.0,
            base_y: 0.0,
            entities: block_entities,
        });
    }

    blocks
}

/// レイヤー名を取得する
fn get_layer_name(jww_doc: &JwwDocument, layer_group: u16, layer: u16) -> String {
    if (layer_group as usize) < 16 && (layer as usize) < 16 {
        let lg = &jww_doc.layer_groups[layer_group as usize];
        let l = &lg.layers[layer as usize];
        if !l.name.is_empty() {
            return l.name.clone();
        }
    }
    format!("{:X}-{:X}", layer_group, layer)
}

/// ブロック名を取得する
fn get_block_name(jww_doc: &JwwDocument, def_number: u32) -> String {
    for bd in &jww_doc.block_defs {
        if bd.number == def_number {
            if !bd.name.is_empty() {
                return bd.name.clone();
            }
            break;
        }
    }
    format!("BLOCK_{}", def_number)
}

/// JWW色コードをDXF ACI値にマッピングする
fn map_color(jww_color: u16) -> i32 {
    match jww_color {
        0 => 0,    // BYLAYER
        1 => 4,    // JWW 水色 -> DXF cyan
        2 => 7,    // JWW 白 -> DXF white
        3 => 3,    // JWW 緑 -> DXF green
        4 => 2,    // JWW 黄色 -> DXF yellow
        5 => 6,    // JWW ピンク -> DXF magenta
        6 => 5,    // JWW 青 -> DXF blue
        7 => 7,    // JWW 黒/白 -> DXF white/black
        8 => 1,    // JWW 赤 -> DXF red
        9 => 8,    // JWW グレー -> DXF gray
        _ => {
            if jww_color >= 100 {
                (jww_color - 100 + 10) as i32
            } else {
                jww_color as i32
            }
        }
    }
}

/// JWW線種をDXF線種名にマッピングする
fn map_line_type(pen_style: u8) -> String {
    match pen_style {
        0 | 1 => "CONTINUOUS",
        2 => "DASHED",
        3 => "DASHDOT",
        4 => "CENTER",
        5 => "DOT",
        6 => "DASHEDX2",
        7 => "DASHDOTX2",
        8 => "CENTERX2",
        9 => "DOTX2",
        _ => "CONTINUOUS",
    }
    .to_string()
}

/// ラジアンを度に変換する
fn rad_to_deg(rad: f64) -> f64 {
    rad * 180.0 / std::f64::consts::PI
}
