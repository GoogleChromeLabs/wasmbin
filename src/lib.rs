#![cfg_attr(feature = "nightly", feature(arbitrary_enum_discriminant, never_type))]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::match_bool,
    clippy::must_use_candidate,
    clippy::module_name_repetitions
)]

use wasmbin_derive::wasmbin_discriminants;

#[macro_use]
pub mod io;
#[macro_use]
pub mod visit;

pub mod builtins;
pub mod indices;
pub mod instructions;
pub mod module;
pub mod sections;
pub mod types;

pub use module::Module;
