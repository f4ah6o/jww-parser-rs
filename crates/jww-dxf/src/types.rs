//! DXF型定義

use serde::{Deserialize, Serialize};

/// DXFドキュメント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// レイヤーリスト
    pub layers: Vec<Layer>,
    /// エンティティリスト
    pub entities: Vec<Entity>,
    /// ブロックリスト
    pub blocks: Vec<Block>,
}

/// DXFレイヤー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// レイヤー名
    pub name: String,
    /// 色番号 (ACI)
    pub color: i32,
    /// 線種名
    pub line_type: String,
    /// 凍結状態
    pub frozen: bool,
    /// ロック状態
    pub locked: bool,
}

/// DXFエンティティ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Entity {
    Line(Line),
    Circle(Circle),
    Arc(Arc),
    Ellipse(Ellipse),
    Point(Point),
    Text(Text),
    Solid(Solid),
    Insert(Insert),
}

/// 直線
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    /// レイヤー名
    pub layer: String,
    /// 色番号
    pub color: i32,
    /// 線種
    pub line_type: String,
    /// 始点X
    pub x1: f64,
    /// 始点Y
    pub y1: f64,
    /// 終点X
    pub x2: f64,
    /// 終点Y
    pub y2: f64,
}

/// 円
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circle {
    /// レイヤー名
    pub layer: String,
    /// 色番号
    pub color: i32,
    /// 線種
    pub line_type: String,
    /// 中心X
    pub center_x: f64,
    /// 中心Y
    pub center_y: f64,
    /// 半径
    pub radius: f64,
}

/// 円弧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arc {
    /// レイヤー名
    pub layer: String,
    /// 色番号
    pub color: i32,
    /// 線種
    pub line_type: String,
    /// 中心X
    pub center_x: f64,
    /// 中心Y
    pub center_y: f64,
    /// 半径
    pub radius: f64,
    /// 開始角度 (度)
    pub start_angle: f64,
    /// 終了角度 (度)
    pub end_angle: f64,
}

/// 楕円
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ellipse {
    /// レイヤー名
    pub layer: String,
    /// 色番号
    pub color: i32,
    /// 線種
    pub line_type: String,
    /// 中心X
    pub center_x: f64,
    /// 中心Y
    pub center_y: f64,
    /// 長軸ベクトルX
    pub major_axis_x: f64,
    /// 長軸ベクトルY
    pub major_axis_y: f64,
    /// 短軸比率
    pub minor_ratio: f64,
    /// 開始パラメータ
    pub start_param: f64,
    /// 終了パラメータ
    pub end_param: f64,
}

/// 点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    /// レイヤー名
    pub layer: String,
    /// 色番号
    pub color: i32,
    /// 線種
    pub line_type: String,
    /// X座標
    pub x: f64,
    /// Y座標
    pub y: f64,
}

/// 文字
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    /// レイヤー名
    pub layer: String,
    /// 色番号
    pub color: i32,
    /// 線種
    pub line_type: String,
    /// 挿入点X
    pub x: f64,
    /// 挿入点Y
    pub y: f64,
    /// 高さ
    pub height: f64,
    /// 回転角度 (度)
    pub rotation: f64,
    /// 文字列内容
    pub content: String,
    /// スタイル名
    pub style: String,
}

/// 塗りつぶし
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solid {
    /// レイヤー名
    pub layer: String,
    /// 色番号
    pub color: i32,
    /// 線種
    pub line_type: String,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub x3: f64,
    pub y3: f64,
    pub x4: f64,
    pub y4: f64,
}

/// ブロック挿入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insert {
    /// レイヤー名
    pub layer: String,
    /// 色番号
    pub color: i32,
    /// 線種
    pub line_type: String,
    /// ブロック名
    pub block_name: String,
    /// 挿入点X
    pub x: f64,
    /// 挿入点Y
    pub y: f64,
    /// Xスケール
    pub scale_x: f64,
    /// Yスケール
    pub scale_y: f64,
    /// 回転角度 (度)
    pub rotation: f64,
}

/// DXFブロック定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// ブロック名
    pub name: String,
    /// 基準点X
    pub base_x: f64,
    /// 基準点Y
    pub base_y: f64,
    /// エンティティリスト
    pub entities: Vec<Entity>,
}
