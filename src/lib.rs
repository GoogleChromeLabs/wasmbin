use thiserror::Error;
use wasmbin_derive::Wasmbin;

mod builtins;

mod indices;
mod types;
mod instructions;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Leb128(#[from] leb128::read::Error),

    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Unsupported discriminant {discriminant} for type {ty}")]
    UnsupportedDiscriminant {
        discriminant: i128,
        ty: &'static str,
    },
}

pub trait WasmbinEncode {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()>;

    fn encode_seq(seq: &[Self], w: &mut impl std::io::Write) -> std::io::Result<()>
    where
        Self: Sized,
    {
        for item in seq {
            item.encode(w)?;
        }
        Ok(())
    }
}

pub trait WasmbinDecode: Sized + WasmbinEncode {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError>;

    fn decode_seq(count: usize, r: &mut impl std::io::Read) -> Result<Vec<Self>, DecodeError> {
        (0..count).map(|_| Self::decode(r)).collect()
    }
}
