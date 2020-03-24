use thiserror::Error;
pub use wasmbin_derive::Wasmbin;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Leb128(#[from] leb128::read::Error),

    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Could not recognise discriminant 0x{discriminant:02X}")]
    UnsupportedDiscriminant { discriminant: u8 },

    #[error("Invalid module magic signature")]
    InvalidMagic,

    #[error("Unrecognized data")]
    UnrecognizedData,
}

pub trait WasmbinEncode {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()>;
}

pub trait WasmbinDecode: Sized {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError>;
}

pub trait WasmbinDecodeWithDiscriminant: WasmbinDecode {
    fn maybe_decode_with_discriminant(
        discriminant: u8,
        r: &mut impl std::io::Read,
    ) -> Result<Option<Self>, DecodeError>;

    fn decode_with_discriminant(
        discriminant: u8,
        r: &mut impl std::io::Read,
    ) -> Result<Self, DecodeError> {
        Self::maybe_decode_with_discriminant(discriminant, r)?
            .ok_or_else(|| DecodeError::UnsupportedDiscriminant { discriminant })
    }

    fn decode_without_discriminant(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Self::decode_with_discriminant(u8::decode(r)?, r)
    }
}