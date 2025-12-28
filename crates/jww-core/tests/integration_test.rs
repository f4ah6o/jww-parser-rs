//! 統合テスト

#[test]
fn test_invalid_signature() {
    let invalid_data = b"Invalid signature";
    let result = jww_core::parse(invalid_data);
    assert!(result.is_err());
    match result {
        Err(jww_core::ParseError::InvalidSignature) => {}
        _ => panic!("Expected InvalidSignature error"),
    }
}

#[test]
fn test_too_short_data() {
    let short_data = b"short";
    let result = jww_core::parse(short_data);
    assert!(result.is_err());
}

#[test]
fn test_valid_jww_signature() {
    // 最小限の有効なJWWデータを作成
    let mut data = Vec::new();
    data.extend_from_slice(b"JwwData.");
    // バージョン (600 = 0x258)
    data.extend_from_slice(&600u32.to_le_bytes());
    // メモ（空文字列）
    data.push(0);
    // 用紙サイズ
    data.extend_from_slice(&0u32.to_le_bytes());
    // レイヤグループ
    data.extend_from_slice(&0u32.to_le_bytes());

    // 16レイヤグループ分のデータ
    for _ in 0..16 {
        data.extend_from_slice(&2u32.to_le_bytes()); // state
        data.extend_from_slice(&0u32.to_le_bytes()); // write_layer
        data.extend_from_slice(&1.0f64.to_le_bytes()); // scale
        data.extend_from_slice(&0u32.to_le_bytes()); // protect
        // 16レイヤ分
        for _ in 0..16 {
            data.extend_from_slice(&2u32.to_le_bytes()); // lay_state
            data.extend_from_slice(&0u32.to_le_bytes()); // lay_protect
        }
    }

    // エンティティリストのパターン（find_entity_list_offset用）
    // パターン: [count WORD] [0xFF 0xFF] [schema WORD] [name_len WORD] ["CDataXXXX"]
    // 注: エンティティリストはファイルの後半にある必要がある
    data.extend_from_slice(&0u16.to_le_bytes()); // count = 0 (空のエンティティリスト)
    data.extend_from_slice(&0xFFFFu16.to_le_bytes()); // new class marker
    data.extend_from_slice(&600u16.to_le_bytes()); // schema (version 600)
    data.extend_from_slice(&8u16.to_le_bytes()); // name_len = 8
    data.extend_from_slice(b"CDataXXXX"); // class name

    // パディングを追加してファイルサイズを増やす（find_entity_list_offsetが探索するため）
    // 実際のJWWファイルではエンティティリストの後にもデータがある
    for _ in 0..100 {
        data.push(0);
    }

    let result = jww_core::parse(&data);
    assert!(result.is_ok(), "parse failed: {:?}", result.err());

    let doc = result.unwrap();
    assert_eq!(doc.version, 600);
    assert_eq!(doc.entities.len(), 0);
}

#[test]
fn test_dxf_conversion() {
    // 最小限のJWWデータを作成
    let mut data = Vec::new();
    data.extend_from_slice(b"JwwData.");
    data.extend_from_slice(&600u32.to_le_bytes());
    data.push(0); // メモ（空）
    data.extend_from_slice(&0u32.to_le_bytes()); // 用紙サイズ
    data.extend_from_slice(&0u32.to_le_bytes()); // レイヤグループ

    // 16レイヤグループ
    for _ in 0..16 {
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&1.0f64.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        for _ in 0..16 {
            data.extend_from_slice(&2u32.to_le_bytes());
            data.extend_from_slice(&0u32.to_le_bytes());
        }
    }

    // エンティティリストのパターン
    data.extend_from_slice(&0u16.to_le_bytes()); // count = 0
    data.extend_from_slice(&0xFFFFu16.to_le_bytes()); // new class marker
    data.extend_from_slice(&600u16.to_le_bytes()); // schema
    data.extend_from_slice(&8u16.to_le_bytes()); // name_len
    data.extend_from_slice(b"CDataXXXX"); // class name

    // パディングを追加
    for _ in 0..100 {
        data.push(0);
    }

    let jww_doc = jww_core::parse(&data).unwrap();
    let dxf_doc = jww_dxf::convert_document(&jww_doc);

    assert_eq!(dxf_doc.layers.len(), 256); // 16 x 16
    assert_eq!(dxf_doc.entities.len(), 0);
    assert_eq!(dxf_doc.blocks.len(), 0);
}

#[test]
fn test_dxf_to_string() {
    // 最小限のJWWデータを作成
    let mut data = Vec::new();
    data.extend_from_slice(b"JwwData.");
    data.extend_from_slice(&600u32.to_le_bytes());
    data.push(0); // メモ（空）
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());

    for _ in 0..16 {
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&1.0f64.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        for _ in 0..16 {
            data.extend_from_slice(&2u32.to_le_bytes());
            data.extend_from_slice(&0u32.to_le_bytes());
        }
    }

    // エンティティリストのパターン
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&0xFFFFu16.to_le_bytes());
    data.extend_from_slice(&600u16.to_le_bytes());
    data.extend_from_slice(&8u16.to_le_bytes());
    data.extend_from_slice(b"CDataXXXX");

    // パディングを追加
    for _ in 0..100 {
        data.push(0);
    }

    let jww_doc = jww_core::parse(&data).unwrap();
    let dxf_doc = jww_dxf::convert_document(&jww_doc);
    let dxf_string = jww_dxf::to_string(&dxf_doc);

    // DXF文字列の基本構造を確認
    assert!(dxf_string.contains("SECTION"));
    assert!(dxf_string.contains("HEADER"));
    assert!(dxf_string.contains("TABLES"));
    assert!(dxf_string.contains("ENTITIES"));
    assert!(dxf_string.contains("EOF"));
}
