//! DXF文字列出力

use crate::types::Document;
use std::fmt::Write;

/// DXFドキュメントを文字列に変換する
pub fn to_string(doc: &Document) -> String {
    let mut output = String::new();

    // ヘッダーセクション
    writeln!(output, "0").unwrap();
    writeln!(output, "SECTION").unwrap();
    writeln!(output, "2").unwrap();
    writeln!(output, "HEADER").unwrap();
    writeln!(output, "0").unwrap();
    writeln!(output, "ENDSEC").unwrap();

    // テーブルセクション
    writeln!(output, "0").unwrap();
    writeln!(output, "SECTION").unwrap();
    writeln!(output, "2").unwrap();
    writeln!(output, "TABLES").unwrap();

    // LTYPEテーブル
    writeln!(output, "0").unwrap();
    writeln!(output, "TABLE").unwrap();
    writeln!(output, "2").unwrap();
    writeln!(output, "LTYPE").unwrap();
    writeln!(output, "70").unwrap();
    writeln!(output, "1").unwrap();
    writeln!(output, "0").unwrap();
    writeln!(output, "LTYPE").unwrap();
    writeln!(output, "2").unwrap();
    writeln!(output, "CONTINUOUS").unwrap();
    writeln!(output, "70").unwrap();
    writeln!(output, "0").unwrap();
    writeln!(output, "3").unwrap();
    writeln!(output, "Solid line").unwrap();
    writeln!(output, "72").unwrap();
    writeln!(output, "65").unwrap();
    writeln!(output, "73").unwrap();
    writeln!(output, "0").unwrap();
    writeln!(output, "40").unwrap();
    writeln!(output, "0.0").unwrap();
    writeln!(output, "0").unwrap();
    writeln!(output, "ENDTAB").unwrap();

    // LAYERテーブル
    writeln!(output, "0").unwrap();
    writeln!(output, "TABLE").unwrap();
    writeln!(output, "2").unwrap();
    writeln!(output, "LAYER").unwrap();
    writeln!(output, "70").unwrap();
    writeln!(output, "{}", doc.layers.len() + 1).unwrap(); // +1 for required layer 0

    // 必須レイヤー "0" (DXF仕様で必須)
    writeln!(output, "0").unwrap();
    writeln!(output, "LAYER").unwrap();
    writeln!(output, "2").unwrap();
    writeln!(output, "0").unwrap();
    writeln!(output, "70").unwrap();
    writeln!(output, "0").unwrap();
    writeln!(output, "62").unwrap();
    writeln!(output, "7").unwrap(); // white/black
    writeln!(output, "6").unwrap();
    writeln!(output, "CONTINUOUS").unwrap();

    for layer in &doc.layers {
        writeln!(output, "0").unwrap();
        writeln!(output, "LAYER").unwrap();
        writeln!(output, "2").unwrap();
        writeln!(output, "{}", layer.name).unwrap();
        writeln!(output, "70").unwrap();
        writeln!(output, "0").unwrap();
        writeln!(output, "62").unwrap();
        writeln!(output, "{}", layer.color).unwrap();
        writeln!(output, "6").unwrap();
        writeln!(output, "{}", layer.line_type).unwrap();
        if layer.frozen {
            writeln!(output, "70").unwrap();
            writeln!(output, "1").unwrap();
        }
        if layer.locked {
            writeln!(output, "70").unwrap();
            writeln!(output, "4").unwrap();
        }
    }

    writeln!(output, "0").unwrap();
    writeln!(output, "ENDTAB").unwrap();

    // テーブルセクション終了
    writeln!(output, "0").unwrap();
    writeln!(output, "ENDSEC").unwrap();

    // ブロックセクション
    if !doc.blocks.is_empty() {
        writeln!(output, "0").unwrap();
        writeln!(output, "SECTION").unwrap();
        writeln!(output, "2").unwrap();
        writeln!(output, "BLOCKS").unwrap();

        for block in &doc.blocks {
            writeln!(output, "0").unwrap();
            writeln!(output, "BLOCK").unwrap();
            writeln!(output, "8").unwrap();
            writeln!(output, "0").unwrap();
            writeln!(output, "2").unwrap();
            writeln!(output, "{}", block.name).unwrap();
            writeln!(output, "70").unwrap();
            writeln!(output, "0").unwrap();
            writeln!(output, "10").unwrap();
            writeln!(output, "{}", block.base_x).unwrap();
            writeln!(output, "20").unwrap();
            writeln!(output, "{}", block.base_y).unwrap();

            // ブロック内のエンティティ
            for entity in &block.entities {
                write_entity(&mut output, entity);
            }

            writeln!(output, "0").unwrap();
            writeln!(output, "ENDBLK").unwrap();
        }

        writeln!(output, "0").unwrap();
        writeln!(output, "ENDSEC").unwrap();
    }

    // エンティティセクション
    writeln!(output, "0").unwrap();
    writeln!(output, "SECTION").unwrap();
    writeln!(output, "2").unwrap();
    writeln!(output, "ENTITIES").unwrap();

    for entity in &doc.entities {
        write_entity(&mut output, entity);
    }

    writeln!(output, "0").unwrap();
    writeln!(output, "ENDSEC").unwrap();

    // ファイル終了
    writeln!(output, "0").unwrap();
    writeln!(output, "EOF").unwrap();

    output
}

/// エンティティをDXF形式で出力する
fn write_entity(output: &mut String, entity: &crate::types::Entity) {
    use crate::types::Entity;

    match entity {
        Entity::Line(line) => {
            writeln!(output, "0").unwrap();
            writeln!(output, "LINE").unwrap();
            writeln!(output, "8").unwrap();
            writeln!(output, "{}", line.layer).unwrap();
            writeln!(output, "62").unwrap();
            writeln!(output, "{}", line.color).unwrap();
            writeln!(output, "6").unwrap();
            writeln!(output, "{}", line.line_type).unwrap();
            writeln!(output, "10").unwrap();
            writeln!(output, "{}", line.x1).unwrap();
            writeln!(output, "20").unwrap();
            writeln!(output, "{}", line.y1).unwrap();
            writeln!(output, "11").unwrap();
            writeln!(output, "{}", line.x2).unwrap();
            writeln!(output, "21").unwrap();
            writeln!(output, "{}", line.y2).unwrap();
        }

        Entity::Circle(circle) => {
            writeln!(output, "0").unwrap();
            writeln!(output, "CIRCLE").unwrap();
            writeln!(output, "8").unwrap();
            writeln!(output, "{}", circle.layer).unwrap();
            writeln!(output, "62").unwrap();
            writeln!(output, "{}", circle.color).unwrap();
            writeln!(output, "6").unwrap();
            writeln!(output, "{}", circle.line_type).unwrap();
            writeln!(output, "10").unwrap();
            writeln!(output, "{}", circle.center_x).unwrap();
            writeln!(output, "20").unwrap();
            writeln!(output, "{}", circle.center_y).unwrap();
            writeln!(output, "40").unwrap();
            writeln!(output, "{}", circle.radius).unwrap();
        }

        Entity::Arc(arc) => {
            writeln!(output, "0").unwrap();
            writeln!(output, "ARC").unwrap();
            writeln!(output, "8").unwrap();
            writeln!(output, "{}", arc.layer).unwrap();
            writeln!(output, "62").unwrap();
            writeln!(output, "{}", arc.color).unwrap();
            writeln!(output, "6").unwrap();
            writeln!(output, "{}", arc.line_type).unwrap();
            writeln!(output, "10").unwrap();
            writeln!(output, "{}", arc.center_x).unwrap();
            writeln!(output, "20").unwrap();
            writeln!(output, "{}", arc.center_y).unwrap();
            writeln!(output, "40").unwrap();
            writeln!(output, "{}", arc.radius).unwrap();
            writeln!(output, "50").unwrap();
            writeln!(output, "{}", arc.start_angle).unwrap();
            writeln!(output, "51").unwrap();
            writeln!(output, "{}", arc.end_angle).unwrap();
        }

        Entity::Ellipse(ellipse) => {
            writeln!(output, "0").unwrap();
            writeln!(output, "ELLIPSE").unwrap();
            writeln!(output, "8").unwrap();
            writeln!(output, "{}", ellipse.layer).unwrap();
            writeln!(output, "62").unwrap();
            writeln!(output, "{}", ellipse.color).unwrap();
            writeln!(output, "6").unwrap();
            writeln!(output, "{}", ellipse.line_type).unwrap();
            writeln!(output, "10").unwrap();
            writeln!(output, "{}", ellipse.center_x).unwrap();
            writeln!(output, "20").unwrap();
            writeln!(output, "{}", ellipse.center_y).unwrap();
            writeln!(output, "11").unwrap();
            writeln!(output, "{}", ellipse.major_axis_x).unwrap();
            writeln!(output, "21").unwrap();
            writeln!(output, "{}", ellipse.major_axis_y).unwrap();
            writeln!(output, "40").unwrap();
            writeln!(output, "{}", ellipse.minor_ratio).unwrap();
            writeln!(output, "41").unwrap();
            writeln!(output, "{}", ellipse.start_param).unwrap();
            writeln!(output, "42").unwrap();
            writeln!(output, "{}", ellipse.end_param).unwrap();
        }

        Entity::Point(point) => {
            writeln!(output, "0").unwrap();
            writeln!(output, "POINT").unwrap();
            writeln!(output, "8").unwrap();
            writeln!(output, "{}", point.layer).unwrap();
            writeln!(output, "62").unwrap();
            writeln!(output, "{}", point.color).unwrap();
            writeln!(output, "6").unwrap();
            writeln!(output, "{}", point.line_type).unwrap();
            writeln!(output, "10").unwrap();
            writeln!(output, "{}", point.x).unwrap();
            writeln!(output, "20").unwrap();
            writeln!(output, "{}", point.y).unwrap();
        }

        Entity::Text(text) => {
            writeln!(output, "0").unwrap();
            writeln!(output, "TEXT").unwrap();
            writeln!(output, "8").unwrap();
            writeln!(output, "{}", text.layer).unwrap();
            writeln!(output, "62").unwrap();
            writeln!(output, "{}", text.color).unwrap();
            writeln!(output, "6").unwrap();
            writeln!(output, "{}", text.line_type).unwrap();
            writeln!(output, "10").unwrap();
            writeln!(output, "{}", text.x).unwrap();
            writeln!(output, "20").unwrap();
            writeln!(output, "{}", text.y).unwrap();
            writeln!(output, "40").unwrap();
            writeln!(output, "{}", text.height).unwrap();
            writeln!(output, "50").unwrap();
            writeln!(output, "{}", text.rotation).unwrap();
            writeln!(output, "1").unwrap();
            writeln!(output, "{}", text.content).unwrap();
            writeln!(output, "7").unwrap();
            writeln!(output, "{}", text.style).unwrap();
        }

        Entity::Solid(solid) => {
            writeln!(output, "0").unwrap();
            writeln!(output, "SOLID").unwrap();
            writeln!(output, "8").unwrap();
            writeln!(output, "{}", solid.layer).unwrap();
            writeln!(output, "62").unwrap();
            writeln!(output, "{}", solid.color).unwrap();
            writeln!(output, "6").unwrap();
            writeln!(output, "{}", solid.line_type).unwrap();
            writeln!(output, "10").unwrap();
            writeln!(output, "{}", solid.x1).unwrap();
            writeln!(output, "20").unwrap();
            writeln!(output, "{}", solid.y1).unwrap();
            writeln!(output, "11").unwrap();
            writeln!(output, "{}", solid.x2).unwrap();
            writeln!(output, "21").unwrap();
            writeln!(output, "{}", solid.y2).unwrap();
            writeln!(output, "12").unwrap();
            writeln!(output, "{}", solid.x3).unwrap();
            writeln!(output, "22").unwrap();
            writeln!(output, "{}", solid.y3).unwrap();
            writeln!(output, "13").unwrap();
            writeln!(output, "{}", solid.x4).unwrap();
            writeln!(output, "23").unwrap();
            writeln!(output, "{}", solid.y4).unwrap();
        }

        Entity::Insert(insert) => {
            writeln!(output, "0").unwrap();
            writeln!(output, "INSERT").unwrap();
            writeln!(output, "8").unwrap();
            writeln!(output, "{}", insert.layer).unwrap();
            writeln!(output, "62").unwrap();
            writeln!(output, "{}", insert.color).unwrap();
            writeln!(output, "6").unwrap();
            writeln!(output, "{}", insert.line_type).unwrap();
            writeln!(output, "2").unwrap();
            writeln!(output, "{}", insert.block_name).unwrap();
            writeln!(output, "10").unwrap();
            writeln!(output, "{}", insert.x).unwrap();
            writeln!(output, "20").unwrap();
            writeln!(output, "{}", insert.y).unwrap();
            writeln!(output, "41").unwrap();
            writeln!(output, "{}", insert.scale_x).unwrap();
            writeln!(output, "42").unwrap();
            writeln!(output, "{}", insert.scale_y).unwrap();
            writeln!(output, "50").unwrap();
            writeln!(output, "{}", insert.rotation).unwrap();
        }
    }
}
