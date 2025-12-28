use serde::{Deserialize, Serialize};

/// JWWドキュメント全体を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// JWWファイルフォーマットバージョン (例: 351 for Ver.3.51, 420 for Ver.4.20)
    pub version: u32,

    /// ファイルメモ/説明
    pub memo: String,

    /// 用紙サイズ: 0-4でA0-A4、8で2A、9で3Aなど
    pub paper_size: u32,

    /// 現在の書き込みレイヤグループ (0-15)
    pub write_layer_group: u32,

    /// 16個のレイヤグループ（各グループに16レイヤ）
    pub layer_groups: [LayerGroup; 16],

    /// 図面エンティティ（線、円弧、文字など）
    pub entities: Vec<Entity>,

    /// ブロック定義
    pub block_defs: Vec<BlockDef>,
}

/// レイヤグループ (JWW: レイヤグループ)
///
/// JWWは16個のレイヤグループを持ち、各グループに16個のレイヤを持つ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerGroup {
    /// レイヤグループの状態: 0=非表示, 1=表示のみ, 2=編集可能, 3=書込モード
    pub state: u32,

    /// グループ内の現在の書き込みレイヤ (0-15)
    pub write_layer: u32,

    /// スケール分母 (例: 100.0で1:100)
    pub scale: f64,

    /// 保護フラグ
    pub protect: u32,

    /// 16個のレイヤ
    pub layers: [Layer; 16],

    /// レイヤグループ名
    pub name: String,
}

/// 個別レイヤ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// レイヤの状態: 0=非表示, 1=表示のみ, 2=編集可能, 3=書込モード
    pub state: u32,

    /// 保護フラグ
    pub protect: u32,

    /// レイヤ名
    pub name: String,
}

/// 全エンティティに共通する属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityBase {
    /// 曲線属性番号 (線種グループ)
    pub group: u32,

    /// 線種番号
    pub pen_style: u8,

    /// 線色番号 (1-9は基本色、拡張値はSXF色)
    pub pen_color: u16,

    /// 線幅 (Ver.3.51以降で利用可能)
    pub pen_width: u16,

    /// レイヤ番号 (0-15)
    pub layer: u16,

    /// レイヤグループ番号 (0-15)
    pub layer_group: u16,

    /// 各種属性フラグ
    pub flag: u16,
}

/// エンティティ種別
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Entity {
    Line(Line),
    Arc(Arc),
    Point(Point),
    Text(Text),
    Solid(Solid),
    Block(Block),
}

impl Entity {
    /// エンティティの基本属性を返す
    pub fn base(&self) -> &EntityBase {
        match self {
            Entity::Line(e) => &e.base,
            Entity::Arc(e) => &e.base,
            Entity::Point(e) => &e.base,
            Entity::Text(e) => &e.base,
            Entity::Solid(e) => &e.base,
            Entity::Block(e) => &e.base,
        }
    }

    /// エンティティの基本属性を可変で返す
    pub fn base_mut(&mut self) -> &mut EntityBase {
        match self {
            Entity::Line(e) => &mut e.base,
            Entity::Arc(e) => &mut e.base,
            Entity::Point(e) => &mut e.base,
            Entity::Text(e) => &mut e.base,
            Entity::Solid(e) => &mut e.base,
            Entity::Block(e) => &mut e.base,
        }
    }

    /// エンティティタイプ名を返す
    pub fn type_name(&self) -> &'static str {
        match self {
            Entity::Line(_) => "LINE",
            Entity::Arc(e) => {
                if e.is_full_circle {
                    "CIRCLE"
                } else {
                    "ARC"
                }
            }
            Entity::Point(_) => "POINT",
            Entity::Text(_) => "TEXT",
            Entity::Solid(_) => "SOLID",
            Entity::Block(_) => "BLOCK",
        }
    }
}

/// 直線エンティティ (JWWクラス: CDataSen)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    #[serde(flatten)]
    pub base: EntityBase,
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
}

/// 円弧/円エンティティ (JWWクラス: CDataEnko)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arc {
    #[serde(flatten)]
    pub base: EntityBase,
    /// 中心X座標
    pub center_x: f64,
    /// 中心Y座標
    pub center_y: f64,
    /// 半径（楕円の場合は長軸半径）
    pub radius: f64,
    /// 開始角度 (ラジアン)
    pub start_angle: f64,
    /// 円弧角度 (ラジアン)
    pub arc_angle: f64,
    /// 回転角度 (ラジアン、楕円の場合)
    pub tilt_angle: f64,
    /// 扁平率 (1.0は真円、それ以外は楕円)
    pub flatness: f64,
    /// 完全円かどうか
    pub is_full_circle: bool,
}

/// 点エンティティ (JWWクラス: CDataTen)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    #[serde(flatten)]
    pub base: EntityBase,
    pub x: f64,
    pub y: f64,
    /// 仮点かどうか
    pub is_temporary: bool,
    /// 点マーカー種別コード
    pub code: u32,
    /// 回転角度
    pub angle: f64,
    /// スケール
    pub scale: f64,
}

/// 文字エンティティ (JWWクラス: CDataMoji)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    #[serde(flatten)]
    pub base: EntityBase,
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
    /// 文字スタイルフラグ: +10000でイタリック、+20000で太字
    pub text_type: u32,
    pub size_x: f64,
    pub size_y: f64,
    /// 文字間隔
    pub spacing: f64,
    /// 回転角度 (度)
    pub angle: f64,
    /// フォント名
    pub font_name: String,
    /// 文字列内容
    pub content: String,
}

/// 塗りつぶしエンティティ (JWWクラス: CDataSolid)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solid {
    #[serde(flatten)]
    pub base: EntityBase,
    pub point1_x: f64,
    pub point1_y: f64,
    pub point2_x: f64,
    pub point2_y: f64,
    pub point3_x: f64,
    pub point3_y: f64,
    pub point4_x: f64,
    pub point4_y: f64,
    /// 色 (pen_color == 10の時使用)
    pub color: u32,
}

/// ブロック挿入エンティティ (JWWクラス: CDataBlock)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    #[serde(flatten)]
    pub base: EntityBase,
    /// 挿入基準点X座標
    pub ref_x: f64,
    /// 挿入基準点Y座標
    pub ref_y: f64,
    /// X方向スケール
    pub scale_x: f64,
    /// Y方向スケール
    pub scale_y: f64,
    /// 回転角度 (ラジアン)
    pub rotation: f64,
    /// 参照先ブロック定義番号
    pub def_number: u32,
}

/// ブロック定義 (JWWクラス: CDataList)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDef {
    #[serde(flatten)]
    pub base: EntityBase,
    /// ブロック定義番号
    pub number: u32,
    /// 参照されているかどうか
    pub is_referenced: bool,
    /// ブロック名
    pub name: String,
    /// ブロックを構成するエンティティ
    pub entities: Vec<Entity>,
}

impl Default for LayerGroup {
    fn default() -> Self {
        Self {
            state: 2,
            write_layer: 0,
            scale: 1.0,
            protect: 0,
            layers: std::array::from_fn(|_| Layer::default()),
            name: String::new(),
        }
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            state: 2,
            protect: 0,
            name: String::new(),
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self {
            version: 0,
            memo: String::new(),
            paper_size: 0,
            write_layer_group: 0,
            layer_groups: std::array::from_fn(|_| LayerGroup::default()),
            entities: Vec::new(),
            block_defs: Vec::new(),
        }
    }
}
