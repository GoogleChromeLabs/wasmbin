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

use crate::builtins::{Lazy, UnparsedBytes, WasmbinCountable};
use crate::io::{Decode, DecodeAsIter, DecodeError, DecodeErrorKind, Encode};
use crate::visit::Visit;

impl Encode for [u8] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.len().encode(w)?;
        w.write_all(self)
    }
}

impl Decode for Vec<u8> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let size = u32::decode(r)?;
        let mut taken = std::io::Read::take(r, size.into());
        let bytes = UnparsedBytes::decode(&mut taken)?.bytes;
        if taken.limit() != 0 {
            return Err(DecodeErrorKind::UnrecognizedData.into());
        }
        Ok(bytes)
    }
}

/// A length-prefixed blob that can be skipped over during decoding.
#[derive(Default, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Blob<T: Decode> {
    /// Lazily-decoded contents of the blob.
    pub contents: Lazy<T>,
}

impl<T: Decode> std::ops::Deref for Blob<T> {
    type Target = Lazy<T>;

    fn deref(&self) -> &Self::Target {
        &self.contents
    }
}

impl<T: Decode> std::ops::DerefMut for Blob<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.contents
    }
}

impl<T: Decode + std::fmt::Debug> std::fmt::Debug for Blob<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.contents.fmt(f)
    }
}

impl<T: Decode + Encode> Encode for Blob<T> {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        let mut buf;
        let raw: &[u8] = match self.contents.try_as_raw() {
            Ok(raw) => raw,
            Err(value) => {
                buf = <Vec<u8>>::new();
                value.encode(&mut buf)?;
                &buf
            }
        };
        raw.encode(w)
    }
}

impl<T: Decode> Decode for Blob<T> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let raw = <Vec<u8>>::decode(r)?;
        Ok(Self {
            contents: Lazy::from_raw(UnparsedBytes { bytes: raw }),
        })
    }
}

impl<T: Decode + WasmbinCountable> WasmbinCountable for Blob<T> {}

impl<T: Decode> From<T> for Blob<T> {
    fn from(value: T) -> Self {
        Blob {
            contents: value.into(),
        }
    }
}

impl<T: DecodeAsIter> IntoIterator for Blob<Vec<T>> {
    type Item = Result<T, DecodeError>;
    type IntoIter = super::lazy::LazyIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.contents.into_iter()
    }
}
