use std::io;

/// JWWファイルパース時のエラー型
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// JWWシグネチャが無効
    #[error("invalid JWW signature: expected 'JwwData.'")]
    InvalidSignature,

    /// サポートしていないJWWバージョン
    #[error("unsupported JWW version: {0}")]
    UnsupportedVersion(u32),

    /// 不明なクラスPID
    #[error("unknown class PID: {0}")]
    UnknownClassPid(u32),

    /// 不明なエンティティクラス
    #[error("unknown entity class: {0}")]
    UnknownEntityClass(String),

    /// IOエラー
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// その他のエラー
    #[error("{0}")]
    Other(String),
}

/// JWWパース結果の型エイリアス
pub type Result<T> = std::result::Result<T, ParseError>;
