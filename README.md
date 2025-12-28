# jww-parser-rs

Jw_cad (JWW) ファイルを解析し、DXF 形式へ変換するRustライブラリ。

## 概要

Go製[jww-parser](https://github.com/f4ah6o/jww-parser)のRust移植版。
WebAssembly (Wasm) 対応を強化し、バイナリサイズの削減を目的としている。

## 機能

- **JWWパーサー**: JWWファイルのバイナリ構造を解析
- **DXFエクスポート**: JWWエンティティをDXF形式に変換
- **Wasm対応**: ブラウザ上での動作を想定

## プロジェクト構成

```
jww-parser-rs/
├── Cargo.toml              # ワークスペース設定
├── crates/
│   ├── jww-core/          # コアJWWパーサークレート
│   │   ├── src/
│   │   │   ├── lib.rs     # パブリックAPI、パーサー
│   │   │   ├── reader.rs  # バイナリリーダー（Shift-JIS対応）
│   │   │   ├── types.rs   # データ構造定義
│   │   │   └── error.rs   # エラー型定義
│   │   └── tests/         # 統合テスト
│   ├── jww-dxf/           # DXF変換クレート
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs   # DXF型定義
│   │       ├── converter.rs # JWW -> DXF変換
│   │       └── writer.rs  # DXF文字列出力
│   └── jww-wasm/          # WASMバインディングクレート
│       └── src/
│           └── lib.rs     # wasm-bindgenエクスポート
```

## 使用方法

### ライブラリとして使用

```rust
use jww_core::parse;

fn main() {
    let data = std::fs::read("example.jww").unwrap();
    let doc = parse(&data).unwrap();

    println!("Version: {}, Entities: {}", doc.version, doc.entities.len());
}
```

### DXF変換

```rust
use jww_core::parse;
use jww_dxf::{convert_document, to_string};

fn main() {
    let data = std::fs::read("example.jww").unwrap();
    let jww_doc = parse(&data).unwrap();
    let dxf_doc = convert_document(&jww_doc);
    let dxf_string = to_string(&dxf_doc);

    std::fs::write("output.dxf", dxf_string).unwrap();
}
```

### Wasmビルド

```bash
cd crates/jww-wasm
wasm-pack build --target web
```

## サポートするエンティティ

- `CDataSen` - 直線
- `CDataEnko` - 円弧/円/楕円
- `CDataTen` - 点
- `CDataMoji` - 文字
- `CDataSolid` - 塗りつぶし
- `CDataBlock` - ブロック挿入
- `CDataSunpou` - 寸法（簡易対応）

## 開発状況

- [x] Cargo workspace構築
- [x] jww-coreクレート実装
- [x] jww-dxfクレート実装
- [x] jww-wasmクレート実装
- [x] 基本的なテスト
- [ ] ブロック定義の完全なパース
- [ ] 実際のJWWファイルでの検証
- [ ] Wasmバイナリサイズ最適化

## ライセンス

AGPL-3.0

## 参照

- [jww-parser (Go版)](https://github.com/f4ah6o/jww-parser)
