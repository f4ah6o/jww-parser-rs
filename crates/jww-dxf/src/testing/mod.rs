//! テスト用ユーティリティモジュール
//!
//! Go版jww-parserとの互換性テストで使用する

mod dxf_parser;

pub use dxf_parser::{parse_dxf_entities, DxfEntity, DxfEntityType};
