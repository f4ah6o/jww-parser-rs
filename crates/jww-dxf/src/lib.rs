//! DXF変換ライブラリ
//!
//! JWWドキュメントをDXF形式に変換する機能を提供する。

mod converter;
mod types;
mod writer;

pub use converter::convert_document;
pub use types::*;
pub use writer::to_string;

// テスト用ユーティリティ（testing feature時のみ使用）
#[cfg(feature = "testing")]
pub mod testing;
