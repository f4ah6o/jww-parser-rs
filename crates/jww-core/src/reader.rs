use std::io::Read;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::error::{ParseError, Result};

/// JWWバイナリデータリーダー
///
/// リトルエンディアン形式でバイナリデータを読み取り、
/// Shift-JIS文字列をUTF-8に変換する機能を提供する。
pub struct Reader<R> {
    inner: R,
    bytes_read: u64,
}

impl<R: Read> Reader<R> {
    /// 新しいリーダーを作成する
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            bytes_read: 0,
        }
    }

    /// シグネチャ "JwwData." を読み取って検証する
    pub fn read_signature(&mut self) -> Result<()> {
        let mut sig = [0u8; 8];
        self.read_exact(&mut sig)?;
        if &sig != b"JwwData." {
            return Err(ParseError::InvalidSignature);
        }
        Ok(())
    }

    /// DWORD (32-bit unsigned int) を読み取る
    pub fn read_dword(&mut self) -> Result<u32> {
        let val = self.inner.read_u32::<LittleEndian>()?;
        self.bytes_read += 4;
        Ok(val)
    }

    /// WORD (16-bit unsigned int) を読み取る
    pub fn read_word(&mut self) -> Result<u16> {
        let val = self.inner.read_u16::<LittleEndian>()?;
        self.bytes_read += 2;
        Ok(val)
    }

    /// BYTE (8-bit unsigned int) を読み取る
    pub fn read_byte(&mut self) -> Result<u8> {
        let val = self.inner.read_u8()?;
        self.bytes_read += 1;
        Ok(val)
    }

    /// Double (64-bit float) を読み取る
    pub fn read_double(&mut self) -> Result<f64> {
        let val = self.inner.read_f64::<LittleEndian>()?;
        self.bytes_read += 8;
        Ok(val)
    }

    /// MFC CString形式で文字列を読み取る
    ///
    /// 文字列フォーマット:
    /// - 長さ < 255: 1バイト長さプレフィックス
    /// - 長さ < 65535: 1バイト 0xFF マーカー + 2バイト長さ
    /// - それ以上: 1バイト 0xFF + 2バイト 0xFFFF + 4バイト長さ
    pub fn read_cstring(&mut self) -> Result<String> {
        let len_byte = self.read_byte()?;

        let length = if len_byte < 0xFF {
            len_byte as u32
        } else {
            let len_word = self.read_word()?;
            if len_word < 0xFFFF {
                len_word as u32
            } else {
                self.read_dword()?
            }
        };

        if length == 0 {
            return Ok(String::new());
        }

        let mut buf = vec![0u8; length as usize];
        self.read_exact(&mut buf)?;

        // Shift-JISからUTF-8に変換
        let (utf8_str, ..) = encoding_rs::SHIFT_JIS.decode(&buf);
        Ok(utf8_str.trim_end_matches('\0').to_string())
    }

    /// 指定したバイト数だけスキップする
    pub fn skip(&mut self, n: usize) -> Result<()> {
        let mut buf = vec![0u8; n];
        self.read_exact(&mut buf)?;
        Ok(())
    }

    /// 正確にバイト列を読み取る
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.inner.read_exact(buf)?;
        self.bytes_read += buf.len() as u64;
        Ok(())
    }

    /// 読み取った合計バイト数を返す
    pub fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    /// 内部リーダーを消費して返す
    pub fn into_inner(self) -> R {
        self.inner
    }
}
