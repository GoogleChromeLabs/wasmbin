use thiserror::Error;
use wasmbin_derive::{Wasmbin, WasmbinCountable};

pub mod builtins;

pub mod indices;
pub mod instructions;
pub mod module;
pub mod sections;
pub mod types;

use builtins::WasmbinCountable;
pub use module::Module;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Leb128(#[from] leb128::read::Error),

    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Could not recognise discriminant 0x{discriminant:02X} for type {ty}")]
    UnsupportedDiscriminant { ty: &'static str, discriminant: u8 },

    #[error("Invalid module magic signature")]
    InvalidMagic,

    #[error("Unrecognized data")]
    UnrecognizedData,
}

pub trait WasmbinEncode {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()>;
}

pub trait WasmbinDecode: Sized + WasmbinEncode {
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError>;
}
