# wasmbin

wasmbin is a library implementing parsing and serialization WebAssembly binaries.

It intends to provide a low-level representation of the WebAssembly module that is fully described by Rust type system rather than smart accessors. It also leverages the said type system in conjunction with custom proc-macros functionality to autogenerate parsing/serialization/visitation code for any complex types (structures and enums).

On the user's side this approach allows any type can be used independently to represent/parse/serialize only part of the module, while on the maintainers' side it trivialises adding and testing new WebAssembly features, boiling such changes down to addition of new fields and variants, without having to add custom implementations too.
