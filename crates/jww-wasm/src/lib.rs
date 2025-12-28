//! WebAssembly bindings for jww-parser
//!
//! JWWファイルをパースし、DXF形式に変換するWASMモジュール

use wasm_bindgen::prelude::*;

// パース結果を返すヘルパー型
#[wasm_bindgen]
pub struct ParseResult {
    ok: bool,
    data: JsValue,
    error: String,
}

#[wasm_bindgen]
impl ParseResult {
    #[wasm_bindgen(getter)]
    pub fn ok(&self) -> bool {
        self.ok
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> JsValue {
        self.data.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> String {
        self.error.clone()
    }
}

/// JWWファイルをパースし、JSON表現を返す
///
/// # 引数
/// * `data` - JWWファイルのバイナリデータ (Uint8Array)
///
/// # 戻り値
/// ParseResult - 成功時はdataフィールドにJSON、失敗時はerrorフィールドにエラーメッセージ
#[wasm_bindgen]
pub fn jww_parse(data: &[u8]) -> ParseResult {
    match jww_core::parse(data) {
        Ok(doc) => {
            match serde_wasm_bindgen::to_value(&doc) {
                Ok(json) => ParseResult {
                    ok: true,
                    data: json,
                    error: String::new(),
                },
                Err(e) => ParseResult {
                    ok: false,
                    data: JsValue::NULL,
                    error: format!("JSON marshal error: {}", e),
                },
            }
        }
        Err(e) => ParseResult {
            ok: false,
            data: JsValue::NULL,
            error: format!("parse error: {}", e),
        },
    }
}

/// JWWファイルをパースし、DXF JSONを返す
///
/// # 引数
/// * `data` - JWWファイルのバイナリデータ (Uint8Array)
///
/// # 戻り値
/// ParseResult - 成功時はdataフィールドにDXF JSON、失敗時はerrorフィールドにエラーメッセージ
#[wasm_bindgen]
pub fn jww_to_dxf(data: &[u8]) -> ParseResult {
    match jww_core::parse(data) {
        Ok(jww_doc) => {
            let dxf_doc = jww_dxf::convert_document(&jww_doc);
            match serde_wasm_bindgen::to_value(&dxf_doc) {
                Ok(json) => ParseResult {
                    ok: true,
                    data: json,
                    error: String::new(),
                },
                Err(e) => ParseResult {
                    ok: false,
                    data: JsValue::NULL,
                    error: format!("JSON marshal error: {}", e),
                },
            }
        }
        Err(e) => ParseResult {
            ok: false,
            data: JsValue::NULL,
            error: format!("parse error: {}", e),
        },
    }
}

/// JWWファイルをパースし、DXF文字列を返す
///
/// # 引数
/// * `data` - JWWファイルのバイナリデータ (Uint8Array)
///
/// # 戻り値
/// ParseResult - 成功時はdataフィールドにDXF文字列、失敗時はerrorフィールドにエラーメッセージ
#[wasm_bindgen]
pub fn jww_to_dxf_string(data: &[u8]) -> ParseResult {
    match jww_core::parse(data) {
        Ok(jww_doc) => {
            let dxf_doc = jww_dxf::convert_document(&jww_doc);
            let dxf_string = jww_dxf::to_string(&dxf_doc);
            ParseResult {
                ok: true,
                data: JsValue::from_str(&dxf_string),
                error: String::new(),
            }
        }
        Err(e) => ParseResult {
            ok: false,
            data: JsValue::NULL,
            error: format!("parse error: {}", e),
        },
    }
}

/// WASMモジュールのバージョンを返す
#[wasm_bindgen]
pub fn jww_get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// デバッグモードを設定
#[wasm_bindgen]
pub fn jww_set_debug(_enabled: bool) {
    // TODO: デバッグモードの実装
}

/// コミットハッシュを返す
#[wasm_bindgen]
pub fn jww_commit_hash() -> String {
    // ビルド時に設定される
    "unknown".to_string()
}
