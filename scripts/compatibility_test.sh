#!/bin/bash
# Go版jww-parserとRust版jww-parser-rsの互換性テストを実行するスクリプト

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
GO_PARSER_DIR="/Users/fu2hito/src/jww/jww-parser"
GO_PARSER_BIN="$GO_PARSER_DIR/bin/jww-parser"

echo "=== Go版jww-parser 互換性テスト ==="
echo ""

# Go版パーサーのビルドチェック
if [ ! -f "$GO_PARSER_BIN" ]; then
    echo "Go版パーサーをビルドしています..."
    cd "$GO_PARSER_DIR"
    go build -o bin/jww-parser ./cmd/jww-parser
    echo "Go版パーサーのビルドが完了しました"
    echo ""
fi

# Rust版のテスト実行
echo "Rust版の互換性テストを実行します..."
cd "$PROJECT_ROOT"
cargo test --test compatibility_test -- --ignored

echo ""
echo "=== 互換性テスト完了 ==="
