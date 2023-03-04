# wasmbin

wasmbin is a library implementing parsing and serialization WebAssembly binaries.

**Announcement blog post:** [wasmbin: a self-generating WebAssembly parser & serializer](https://rreverser.com/wasmbin-yet-another-webassembly-parser-serializer/).

**Public API entry point:** [`Module`](https://docs.rs/wasmbin/latest/wasmbin/module/struct.Module.html) object. From there you can explore the module contents by simply looking up the nested fields.

## Motivation

This crate intends to provide a low-level representation of the WebAssembly module that is fully described by Rust type system rather than smart accessors. It also leverages the said type system in conjunction with custom proc-macros functionality to autogenerate parsing/serialization/visitation code for any complex types (structures and enums).

On the user's side this approach allows any type can be used independently to represent/parse/serialize only part of the module, while on the maintainers' side it trivialises adding and testing new WebAssembly features, boiling such changes down to addition of new fields and variants, without having to add custom implementations too.

In conjunction with the custom [`Lazy<T>`](https://docs.rs/wasmbin/latest/wasmbin/builtins/struct.Lazy.html) wrapper used in `wasmbin` whenever the spec permits efficiently skipping over some contents (e.g. function bodies), it also provides minimally invasive, efficient, "zero-cost" encoding and decoding of WebAssembly modules: during decoding anything that can be skipped, is skipped over lazily, and during encoding only modified parts of the module are re-encoded, while others are copied verbatim from source.
