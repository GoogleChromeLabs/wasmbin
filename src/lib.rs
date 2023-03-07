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

#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::match_bool,
    clippy::must_use_candidate,
    clippy::module_name_repetitions
)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;

#[cfg(not(feature = "arbitrary"))]
use wasmbin_derive::Noop as Arbitrary;

pub mod builtins;
pub mod indices;
pub mod instructions;
pub mod io;
mod module;
pub mod sections;
pub mod types;
pub mod visit;

pub use module::Module;

#[cfg(test)]
fn test_roundtrip<
    T: arbitrary::Arbitrary<'static> + std::fmt::Debug + io::Encode + io::Decode + Eq,
>() -> anyhow::Result<()> {
    use anyhow::Context;

    let arbitrary = T::arbitrary(&mut arbitrary::Unstructured::new(&[]))?;

    let mut buf = Vec::new();
    arbitrary.encode(&mut buf)?;

    (|| -> anyhow::Result<()> {
        let mut slice = &buf[..];
        let decoded = T::decode(&mut slice)?;
        assert_eq!(arbitrary, decoded);
        anyhow::ensure!(
            slice.is_empty(),
            "{} leftover bytes after decoding",
            slice.len()
        );
        Ok(())
    })()
    .with_context(|| {
        format!(
            "Failed roundtrip\nValue: {type_name} {arbitrary:#?}\nRaw bytes: {buf:#X?}",
            type_name = std::any::type_name::<T>()
        )
    })
}
