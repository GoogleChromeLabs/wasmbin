#![cfg_attr(feature = "nightly", feature(arbitrary_enum_discriminant, never_type))]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::match_bool,
    clippy::must_use_candidate,
    clippy::module_name_repetitions
)]

use wasmbin_derive::wasmbin_discriminants;

pub mod builtins;
pub mod indices;
pub mod instructions;
pub mod io;
pub mod module;
pub mod sections;
pub mod types;
pub mod visit;
