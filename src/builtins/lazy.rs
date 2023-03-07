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

use crate::builtins::WasmbinCountable;
use crate::io::{Decode, DecodeError, DecodeErrorKind, Encode};
use crate::visit::{Visit, VisitError};
use custom_debug::Debug as CustomDebug;
use once_cell::sync::OnceCell;
use std::hash::Hash;

#[derive(CustomDebug, Clone)]
enum LazyStatus<T> {
    FromInput {
        #[debug(with = "custom_debug::hexbuf_str")]
        raw: Vec<u8>,
        parsed: OnceCell<T>,
    },
    Output {
        value: T,
    },
}

/// A wrapper around a type that allows it to be lazily decoded.
///
/// This is useful for types that are expensive to decode, but allowed
/// to be skipped over by the spec (e.g. as part of a length-prefixed
/// [`Blob`](crate::builtins::Blob)). During decoding, this type will
/// store the raw bytes of the value, and only decode them when
/// explicitly requested.
///
/// When re-encoding, it will check if the value has ever been accessed mutably,
/// and if so, re-encode it. Otherwise it will do a cheap copy of the original
/// raw bytes.
#[derive(Clone)]
pub struct Lazy<T> {
    status: LazyStatus<T>,
}

impl<T> Lazy<T> {
    /// Create a new undecoded `Lazy` from a raw byte vector.
    pub fn from_raw(raw: Vec<u8>) -> Self {
        Lazy {
            status: LazyStatus::FromInput {
                raw,
                parsed: OnceCell::new(),
            },
        }
    }

    /// Retrieve the raw bytes if the value has not been modified yet.
    pub fn try_as_raw(&self) -> Result<&[u8], &T> {
        match &self.status {
            LazyStatus::FromInput { raw, .. } => Ok(raw),
            LazyStatus::Output { value } => Err(value),
        }
    }
}

impl<T> From<T> for Lazy<T> {
    fn from(value: T) -> Self {
        Lazy {
            status: LazyStatus::Output { value },
        }
    }
}

impl<T: Default> Default for Lazy<T> {
    fn default() -> Self {
        T::default().into()
    }
}

impl<T: Encode> Encode for Lazy<T> {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        match &self.status {
            LazyStatus::FromInput { raw, .. } => raw.encode(w),
            LazyStatus::Output { value } => value.encode(w),
        }
    }
}

impl<T: Decode> Decode for Lazy<T> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Vec::decode(r).map(Self::from_raw)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Lazy<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.status.fmt(f)
    }
}

fn decode_raw<T: Decode>(mut raw: &[u8]) -> Result<T, DecodeError> {
    let value = T::decode(&mut raw)?;
    if !raw.is_empty() {
        return Err(DecodeErrorKind::UnrecognizedData.into());
    }
    Ok(value)
}

impl<T: Decode> Lazy<T> {
    /// Retrieve a reference to the inner value, decoding it if it wasn't already.
    pub fn try_contents(&self) -> Result<&T, DecodeError> {
        match &self.status {
            LazyStatus::FromInput { raw, parsed } => parsed.get_or_try_init(|| decode_raw(raw)),
            LazyStatus::Output { value } => Ok(value),
        }
    }

    /// Retrieve a mutable reference to the inner value, decoding it if it wasn't already.
    ///
    /// This will invalidate the original raw bytes.
    pub fn try_contents_mut(&mut self) -> Result<&mut T, DecodeError> {
        if let LazyStatus::FromInput { raw, parsed } = &mut self.status {
            // We can't trust input and output to match once we obtained a mutable reference,
            // so get the value and change the status to just Output.
            let parsed = std::mem::replace(parsed, OnceCell::new());
            self.status = LazyStatus::Output {
                value: match parsed.into_inner() {
                    Some(value) => value,
                    None => decode_raw(raw)?,
                },
            };
        }
        if let LazyStatus::Output { value } = &mut self.status {
            return Ok(value);
        }
        unreachable!()
    }

    /// Unwrap the inner value, decoding it if it wasn't already.
    pub fn try_into_contents(self) -> Result<T, DecodeError> {
        match self.status {
            LazyStatus::FromInput { raw, parsed } => match parsed.into_inner() {
                Some(value) => Ok(value),
                None => decode_raw(&raw),
            },
            LazyStatus::Output { value } => Ok(value),
        }
    }
}

impl<T: Decode + PartialEq> PartialEq for Lazy<T> {
    fn eq(&self, other: &Self) -> bool {
        if let (LazyStatus::FromInput { raw: raw1, .. }, LazyStatus::FromInput { raw: raw2, .. }) =
            (&self.status, &other.status)
        {
            if raw1 == raw2 {
                return true;
            }
        }
        if let (Ok(value1), Ok(value2)) = (self.try_contents(), other.try_contents()) {
            return value1 == value2;
        }
        false
    }
}

impl<T: Decode + Eq> Eq for Lazy<T> {}

impl<T: Decode + Hash> Hash for Lazy<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.try_contents().ok().hash(state);
    }
}

impl<T: WasmbinCountable> WasmbinCountable for Lazy<T> {}

impl<T: Decode + Visit> Visit for Lazy<T> {
    fn visit_children<'a, VisitT: 'static, E, F: FnMut(&'a VisitT) -> Result<(), E>>(
        &'a self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        match self.try_contents() {
            Ok(contents) => contents.visit_child(f),
            Err(err) => Err(VisitError::LazyDecode(err)),
        }
    }

    fn visit_children_mut<VisitT: 'static, E, F: FnMut(&mut VisitT) -> Result<(), E>>(
        &mut self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        match self.try_contents_mut() {
            Ok(contents) => contents.visit_child_mut(f),
            Err(err) => Err(VisitError::LazyDecode(err)),
        }
    }
}
