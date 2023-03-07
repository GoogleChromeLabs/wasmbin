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

use crate::builtins::{Lazy, WasmbinCountable};
use crate::io::{Decode, DecodeError, DecodeErrorKind, Encode};
use crate::visit::Visit;
use crate::Arbitrary;

/// A length-prefixed blob of bytes.
#[derive(Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct RawBlob<T = Vec<u8>> {
    #[allow(missing_docs)]
    pub contents: T,
}

impl<T> std::ops::Deref for RawBlob<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.contents
    }
}

impl<T> std::ops::DerefMut for RawBlob<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.contents
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for RawBlob<T> {
    fn as_ref(&self) -> &[u8] {
        self.contents.as_ref()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for RawBlob<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.contents.fmt(f)
    }
}

impl<T: AsRef<[u8]>> Encode for RawBlob<T> {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        let bytes = self.contents.as_ref();
        bytes.len().encode(w)?;
        bytes.encode(w)
    }
}

impl<T: Decode> Decode for RawBlob<T> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let size = u32::decode(r)?;
        let mut taken = std::io::Read::take(r, size.into());
        let contents = T::decode(&mut taken)?;
        if taken.limit() != 0 {
            return Err(DecodeErrorKind::UnrecognizedData.into());
        }
        Ok(RawBlob { contents })
    }
}

/// A length-prefixed blob that can be skipped over during decoding.
#[derive(Default, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
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
        let value = match self.contents.try_as_raw() {
            Ok(raw) => return RawBlob { contents: raw }.encode(w),
            Err(value) => value,
        };
        let mut buf;
        buf = Vec::new();
        value.encode(&mut buf)?;
        RawBlob { contents: buf }.encode(w)
    }
}

impl<T: Decode> Decode for Blob<T> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let contents: Lazy<T> = RawBlob::decode(r)?.contents;
        Ok(Self { contents })
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
