//! Go版jww-parserとの互換性テスト
//!
//! このテストはGo版パーサーがビルドされている必要があります
//! 実行方法: cargo test --test compatibility_test -- --ignored

use std::path::PathBuf;
use std::process::Command;

/// Go版パーサーのバイナリパス
fn go_parser_path() -> PathBuf {
    PathBuf::from("/Users/fu2hito/src/jww/jww-parser/bin/jww-parser")
}

/// テストフィクスチャディレクトリ
fn fixtures_dir() -> PathBuf {
    PathBuf::from("/Users/fu2hito/src/jww/jww-parser-rs/tests/fixtures")
}

/// Go版パーサーを実行してDXF出力を取得
fn run_go_parser(jww_path: &PathBuf) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let dxf_path = temp_dir.join(format!("output_{}.dxf", std::process::id()));

    let output = Command::new(go_parser_path())
        .arg("-dxf")  // DXF出力
        .arg("-o")
        .arg(&dxf_path)
        .arg(jww_path)
        .output()
        .map_err(|e| format!("Go版パーサーの実行に失敗: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Go版パーサーがエラーを返しました: {}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    std::fs::read_to_string(&dxf_path)
        .map_err(|e| format!("DXFファイルの読み取りに失敗: {}", e))
}

/// Rust版パーサーでJWWをパースしてDXF出力を取得
fn run_rust_parser(jww_data: &[u8]) -> Result<String, String> {
    let jww_doc = jww_core::parse(jww_data)
        .map_err(|e| format!("Rust版パーサーのエラー: {:?}", e))?;

    let dxf_doc = jww_dxf::convert_document(&jww_doc);
    Ok(jww_dxf::to_string(&dxf_doc))
}

#[test]
#[ignore]
fn test_go_parser_available() {
    // Go版パーサーが利用可能か確認
    let path = go_parser_path();
    assert!(
        path.exists(),
        "Go版パーサーが見つかりません: {}\n先にビルドしてください: cd /Users/fu2hito/src/jww/jww-parser && go build -o bin/jww-parser ./cmd/jww-parser",
        path.display()
    );

    // バージョン確認
    let output = Command::new(path)
        .arg("--help")
        .output()
        .expect("Go版パーサーの実行に失敗");

    eprintln!("Go版パーサーのヘルプ出力:\n{}", String::from_utf8_lossy(&output.stdout));
}

/// 最小限のJWWデータを作成（エンティティなし）
fn create_minimal_jww_data() -> Vec<u8> {
    let mut data = Vec::new();

    // シグネチャ
    data.extend_from_slice(b"JwwData.");

    // バージョン (600 = 6.00)
    data.extend_from_slice(&600u32.to_le_bytes());

    // メモ（空文字列）
    data.push(0);

    // 用紙サイズ
    data.extend_from_slice(&0u32.to_le_bytes());

    // 現在の書き込みレイヤグループ
    data.extend_from_slice(&0u32.to_le_bytes());

    // 16レイヤグループ分のデータ
    for g in 0..16 {
        data.extend_from_slice(&2u32.to_le_bytes()); // state
        data.extend_from_slice(&0u32.to_le_bytes()); // write_layer
        data.extend_from_slice(&1.0f64.to_le_bytes()); // scale

        // レイヤグループ名
        let group_name = format!("Group{}", g);
        data.extend_from_slice((group_name.len() as u32).to_le_bytes().as_ref());
        data.extend_from_slice(group_name.as_bytes());

        data.extend_from_slice(&0u32.to_le_bytes()); // protect

        // 16レイヤ分
        for l in 0..16 {
            data.extend_from_slice(&2u32.to_le_bytes()); // lay_state
            data.extend_from_slice(&0u32.to_le_bytes()); // lay_protect

            // レイヤ名
            let layer_name = format!("{}-{}", g, l);
            data.extend_from_slice((layer_name.len() as u32).to_le_bytes().as_ref());
            data.extend_from_slice(layer_name.as_bytes());
        }
    }

    // エンティティリスト（空）
    data.extend_from_slice(&0u16.to_le_bytes()); // count = 0
    data.extend_from_slice(&0xFFFFu16.to_le_bytes()); // new class marker
    data.extend_from_slice(&600u16.to_le_bytes()); // schema
    data.extend_from_slice(&8u16.to_le_bytes()); // name_len
    data.extend_from_slice(b"CDataXXXX"); // class name

    // パディング
    for _ in 0..100 {
        data.push(0);
    }

    data
}

#[test]
#[ignore]
fn test_compatibility_minimal_jww() {
    // 最小限のJWWデータを作成
    let jww_data = create_minimal_jww_data();

    // 一時ファイルに保存
    let temp_dir = std::env::temp_dir();
    let jww_path = temp_dir.join("test_minimal.jww");
    std::fs::write(&jww_path, &jww_data).expect("一時ファイルの書き込みに失敗");

    eprintln!("テストファイル: {}", jww_path.display());

    // Rust版でパース
    let rust_dxf = run_rust_parser(&jww_data).expect("Rust版パーサーが失敗");

    // Go版でパース
    let go_dxf = run_go_parser(&jww_path).expect("Go版パーサーが失敗");

    // DXFを解析して比較
    let (go_entities, go_layers) = jww_dxf::testing::parse_dxf_entities(&go_dxf);
    let (rust_entities, rust_layers) = jww_dxf::testing::parse_dxf_entities(&rust_dxf);

    eprintln!("Go版: {} エンティティ, {} レイヤー", go_entities.len(), go_layers.len());
    eprintln!("Rust版: {} エンティティ, {} レイヤー", rust_entities.len(), rust_layers.len());

    // エンティティ数の比較（空のJWWなので0）
    assert_eq!(go_entities.len(), rust_entities.len(), "エンティティ数が一致しません");

    // レイヤー数の比較
    // 両方とも必須レイヤー"0"を含む257レイヤー
    assert_eq!(go_layers.len(), 257, "Go版のレイヤー数が257ではありません");
    assert_eq!(rust_layers.len(), 257, "Rust版のレイヤー数が257ではありません");

    // 両方の最初のレイヤーは必須レイヤー"0"
    assert_eq!(go_layers[0], "0", "Go版の最初のレイヤーが0ではありません");
    assert_eq!(rust_layers[0], "0", "Rust版の最初のレイヤーが0ではありません");

    // 残りの256レイヤー名の比較（Go版もRust版も1以降がJWWレイヤー）
    for i in 0..256 {
        assert_eq!(
            go_layers[i + 1], rust_layers[i + 1],
            "レイヤー{}の名前が一致しません: Go={}, Rust={}",
            i, go_layers[i + 1], rust_layers[i + 1]
        );
    }
}

#[test]
fn test_rust_parser_only_minimal() {
    // Go版がなくてもRust版単体でテスト
    let jww_data = create_minimal_jww_data();

    let jww_doc = jww_core::parse(&jww_data).expect("パースに失敗");
    assert_eq!(jww_doc.version, 600);

    let dxf_doc = jww_dxf::convert_document(&jww_doc);
    assert_eq!(dxf_doc.layers.len(), 256);
    assert_eq!(dxf_doc.entities.len(), 0);

    let dxf_string = jww_dxf::to_string(&dxf_doc);
    assert!(dxf_string.contains("SECTION"));
    assert!(dxf_string.contains("ENTITIES"));
    assert!(dxf_string.contains("LAYER"));
    assert!(dxf_string.contains("EOF"));
}
