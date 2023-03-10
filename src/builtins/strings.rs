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

use super::RawBlob;
use crate::io::{Decode, DecodeError, Encode};
use crate::visit::Visit;
use std::convert::TryFrom;

pub type String = bytestring::ByteString;

impl Encode for str {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        RawBlob { contents: self }.encode(w)
    }
}

impl Encode for String {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        str::encode(self, w)
    }
}

impl Decode for String {
    fn decode(r: &mut bytes::Bytes) -> Result<Self, DecodeError> {
        let bytes = <RawBlob<bytes::Bytes>>::decode(r)?;
        Ok(String::try_from(bytes.contents)?)
    }
}

impl Visit for String {}
