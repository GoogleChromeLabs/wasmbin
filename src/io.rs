//! Encoding / decoding traits.

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

#![warn(missing_docs)]

use crate::sections::SectionOrderError;
use thiserror::Error;
pub use wasmbin_derive::Wasmbin;

/// [Decode] error kind.
#[derive(Error, Debug)]
pub enum DecodeErrorKind {
    /// Reading error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// LEB128 decoding error.
    #[error(transparent)]
    Leb128(#[from] leb128::read::Error),

    /// UTF-8 decoding error.
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Unsupported (unknown) enum or struct discriminant.
    #[error("Could not recognise discriminant 0x{discriminant:X} for type {ty}")]
    UnsupportedDiscriminant {
        /// The fully-qualified type name.
        ty: &'static str,
        /// Encountered discriminant.
        discriminant: i128,
    },

    /// Invalid module magic signature.
    #[error("Invalid module magic signature [{actual:02X?}]")]
    InvalidMagic {
        /// The actual byte sequence encountered instead of the expected magic signature.
        actual: [u8; 8],
    },

    /// Unrecognized data at the end of a stream or a [`Blob`](crate::builtins::Blob).
    #[error("Unrecognized data")]
    UnrecognizedData,

    /// Encountered section in the wrong position among others.
    #[error(transparent)]
    SectionOutOfOrder(#[from] SectionOrderError),
}

#[derive(Debug)]
pub(crate) enum PathItem {
    Name(&'static str),
    Index(usize),
    Variant(&'static str),
}

/// Decoding error with attached property path.
#[derive(Error, Debug)]
pub struct DecodeError {
    path: Vec<PathItem>,

    /// The kind of error that occurred.
    #[source]
    pub kind: DecodeErrorKind,
}

impl DecodeError {
    pub(crate) fn unsupported_discriminant<T: Decode>(discriminant: impl Into<i128>) -> Self {
        DecodeErrorKind::UnsupportedDiscriminant {
            ty: std::any::type_name::<T>(),
            discriminant: discriminant.into(),
        }
        .into()
    }
}

impl DecodeError {
    pub(crate) fn in_path(mut self, item: PathItem) -> Self {
        self.path.push(item);
        self
    }
}

impl<E: Into<DecodeErrorKind>> From<E> for DecodeError {
    fn from(err: E) -> DecodeError {
        DecodeError {
            path: vec![],
            kind: err.into(),
        }
    }
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("(root)")?;
        for item in self.path.iter().rev() {
            match *item {
                PathItem::Name(name) => write!(f, ".{name}"),
                PathItem::Index(index) => write!(f, "[{index}]"),
                PathItem::Variant(variant) => write!(f, ":<{variant}>"),
            }?;
        }
        write!(f, ": {}", self.kind)
    }
}

impl From<std::num::TryFromIntError> for DecodeErrorKind {
    fn from(_err: std::num::TryFromIntError) -> Self {
        DecodeErrorKind::Leb128(leb128::read::Error::Overflow)
    }
}

impl From<std::convert::Infallible> for DecodeErrorKind {
    fn from(err: std::convert::Infallible) -> Self {
        match err {}
    }
}

/// A trait for types that can be encoded into a binary stream.
pub trait Encode {
    /// Encodes the value into the given writer.
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()>;
}

/// A trait for types that can be decoded from a binary stream.
pub trait Decode: Sized {
    /// Decodes the value from the given reader.
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError>;
}

macro_rules! encode_decode_as {
    ($ty:ty, {
        $($lhs:tt <=> $rhs:tt,)*
    } $(, |$other:pat_param| $other_handler:expr)?) => {
        impl $crate::io::Encode for $ty {
            #[allow(unused_parens)]
            fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                match *self {
                    $($lhs => $rhs,)*
                }.encode(w)
            }
        }

        impl $crate::io::Decode for $ty {
            #[allow(unused_parens)]
            fn decode(r: &mut impl std::io::Read) -> Result<Self, $crate::io::DecodeError> {
                Ok(match $crate::io::Decode::decode(r)? {
                    $($rhs => $lhs,)*
                    $($other => return $other_handler)?
                })
            }
        }
    };
}
pub(crate) use encode_decode_as;

/// A [`Decode`] sub-trait for types that have a discriminant (usually enums).
pub trait DecodeWithDiscriminant: Decode {
    /// The discriminant representation.
    type Discriminant: Decode + Copy + Into<i128>;

    /// Decodes the value from the given reader, if the discriminant matches.
    ///
    /// Returns `Ok(None)` if the discriminant does not match.
    ///
    /// This allows to try decoding multiple types with the same discriminant
    /// without advancing the reader position.
    fn maybe_decode_with_discriminant(
        discriminant: Self::Discriminant,
        r: &mut impl std::io::Read,
    ) -> Result<Option<Self>, DecodeError>;

    /// Decodes the value from the given reader, if the discriminant matches.
    ///
    /// Returns an error if the discriminant does not match.
    fn decode_with_discriminant(
        discriminant: Self::Discriminant,
        r: &mut impl std::io::Read,
    ) -> Result<Self, DecodeError> {
        Self::maybe_decode_with_discriminant(discriminant, r)?
            .ok_or_else(|| DecodeError::unsupported_discriminant::<Self>(discriminant))
    }

    /// Decodes this value fully, including the discriminant.
    ///
    /// This method is intended to be used as an implementation for [`Decode::decode`].
    fn decode_without_discriminant(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Self::decode_with_discriminant(Self::Discriminant::decode(r)?, r)
    }
}
