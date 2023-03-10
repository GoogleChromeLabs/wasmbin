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

#![cfg_attr(feature = "nightly", feature(doc_auto_cfg))]
#![warn(
    clippy::all,
    clippy::pedantic,
    unused_qualifications,
    unused_macros,
    unused_lifetimes,
    single_use_lifetimes
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::match_bool,
    clippy::must_use_candidate,
    clippy::module_name_repetitions
)]
#![doc = include_str!("../README.md")]

pub mod builtins;
pub mod indices;
pub mod instructions;
pub mod io;
mod module;
pub mod sections;
pub mod types;
pub mod visit;

pub use module::Module;
