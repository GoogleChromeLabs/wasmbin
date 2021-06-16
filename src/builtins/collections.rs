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

use crate::io::{Decode, DecodeError, Encode, PathItem};

pub use wasmbin_derive::WasmbinCountable;
pub trait WasmbinCountable {}

impl<T: WasmbinCountable + Encode> Encode for [T] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.len().encode(w)?;
        for item in self {
            item.encode(w)?;
        }
        Ok(())
    }
}

impl<T> Encode for Vec<T>
where
    [T]: Encode,
{
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.as_slice().encode(w)
    }
}

impl<T: WasmbinCountable + Decode> Decode for Vec<T> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let count = usize::decode(r)?;
        (0..count)
            .map(|i| T::decode(r).map_err(move |err| err.in_path(PathItem::Index(i))))
            .collect()
    }
}

impl_visit_for_iter!(Vec<T>);
impl_visit_for_iter!(Option<T>);
