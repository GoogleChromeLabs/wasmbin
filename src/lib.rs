use thiserror::Error;
use wasmbin_derive::Wasmbin;

mod builtins;

mod indices;
mod instructions;
mod sections;
mod types;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Leb128(#[from] leb128::read::Error),

    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Could not recognise discriminant for type {ty}")]
    UnsupportedDiscriminant { ty: &'static str },

    #[error("Invalid module magic signature")]
    InvalidMagic,
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
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError>;

    fn decode_seq(count: u32, r: &mut impl std::io::BufRead) -> Result<Vec<Self>, DecodeError> {
        (0..count).map(|_| Self::decode(r)).collect()
    }
}
