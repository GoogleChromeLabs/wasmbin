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

use crate::io::{Decode, DecodeError, Encode};
use crate::visit::Visit;
use bytes::Bytes;
use std::convert::TryFrom;
use try_buf::TryBuf;

impl<const N: usize> Decode for [u8; N] {
    fn decode(r: &mut bytes::Bytes) -> Result<Self, DecodeError> {
        let mut dest = [0_u8; N];
        r.try_copy_to_slice(&mut dest)?;
        Ok(dest)
    }
}

impl Encode for u8 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        std::slice::from_ref(self).encode(w)
    }
}

impl Encode for [u8] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(self)
    }
}

impl Decode for u8 {
    fn decode(r: &mut bytes::Bytes) -> Result<Self, DecodeError> {
        Ok(r.try_get_u8()?)
    }
}

impl Decode for Option<u8> {
    fn decode(r: &mut bytes::Bytes) -> Result<Self, DecodeError> {
        Ok(r.try_get_u8().ok())
    }
}

impl Decode for Bytes {
    fn decode(r: &mut bytes::Bytes) -> Result<Self, DecodeError> {
        Ok(std::mem::take(r))
    }
}

impl Visit for u8 {}

macro_rules! def_integer {
    ($ty:ident, $leb128_method:ident) => {
        impl Encode for $ty {
            fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                leb128::write::$leb128_method(w, (*self).into()).map(|_| ())
            }
        }

        impl Decode for $ty {
            fn decode(r: &mut bytes::Bytes) -> Result<Self, DecodeError> {
                const LIMIT: usize = (std::mem::size_of::<$ty>() * 8 / 7) + 1;

                let r = bytes::Buf::take(r, LIMIT);
                let as_64 = leb128::read::$leb128_method(&mut bytes::Buf::reader(r))?;
                let res = Self::try_from(as_64)?;

                Ok(res)
            }
        }

        impl Visit for $ty {}
    };
}

def_integer!(u32, unsigned);
def_integer!(i32, signed);
def_integer!(u64, unsigned);
def_integer!(i64, signed);

impl Encode for usize {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        match u32::try_from(*self) {
            Ok(v) => v.encode(w),
            Err(err) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err)),
        }
    }
}

impl Decode for usize {
    fn decode(r: &mut bytes::Bytes) -> Result<Self, DecodeError> {
        Ok(usize::try_from(u32::decode(r)?)?)
    }
}

impl Visit for usize {}
