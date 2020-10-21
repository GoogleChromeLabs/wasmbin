// Copyright 2020 Google Inc. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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

    #[error("Could not recognise discriminant 0x{discriminant:X} for type {ty}")]
    UnsupportedDiscriminant {
        ty: &'static str,
        discriminant: i128,
    },

    #[error("Invalid module magic signature [{actual:02X?}]")]
    InvalidMagic { actual: [u8; 8] },

    #[error("Unrecognized data")]
    UnrecognizedData,

    #[error("Section out of order: {current:?} after {prev:?}")]
    SectionOutOfOrder {
        current: crate::sections::Kind,
        prev: crate::sections::Kind,
    },
}

impl From<std::num::TryFromIntError> for DecodeError {
    fn from(_err: std::num::TryFromIntError) -> Self {
        DecodeError::Leb128(leb128::read::Error::Overflow)
    }
}

impl From<std::convert::Infallible> for DecodeError {
    fn from(err: std::convert::Infallible) -> Self {
        match err {}
    }
}

pub trait Encode {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()>;
}

pub trait Decode: Sized {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError>;
}

macro_rules! encode_decode_as {
    ($ty:ty, {
        $($lhs:tt <=> $rhs:tt,)*
    } $(, |$other:pat| $other_handler:expr)?) => {
        impl crate::io::Encode for $ty {
            #[allow(unused_parens)]
            fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                match *self {
                    $($lhs => $rhs,)*
                }.encode(w)
            }
        }

        impl crate::io::Decode for $ty {
            #[allow(unused_parens)]
            fn decode(r: &mut impl std::io::Read) -> Result<Self, crate::io::DecodeError> {
                Ok(match crate::io::Decode::decode(r)? {
                    $($rhs => $lhs,)*
                    $($other => return $other_handler)?
                })
            }
        }
    };
}

pub trait DecodeWithDiscriminant: Decode {
    const NAME: &'static str;
    type Discriminant: Decode + Copy + Into<i128>;

    fn maybe_decode_with_discriminant(
        discriminant: Self::Discriminant,
        r: &mut impl std::io::Read,
    ) -> Result<Option<Self>, DecodeError>;

    fn decode_with_discriminant(
        discriminant: Self::Discriminant,
        r: &mut impl std::io::Read,
    ) -> Result<Self, DecodeError> {
        Self::maybe_decode_with_discriminant(discriminant, r)?.ok_or_else(|| {
            DecodeError::UnsupportedDiscriminant {
                ty: Self::NAME,
                discriminant: discriminant.into(),
            }
        })
    }

    fn decode_without_discriminant(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Self::decode_with_discriminant(Self::Discriminant::decode(r)?, r)
    }
}
