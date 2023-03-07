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

#![warn(missing_docs)]

use crate::builtins::Blob;
use crate::io::{encode_decode_as, Decode, DecodeError, DecodeErrorKind, Encode, Wasmbin};
use crate::sections::{Section, StdPayload};
use crate::visit::Visit;
use std::cmp::Ordering;

const MAGIC_AND_VERSION: [u8; 8] = [b'\0', b'a', b's', b'm', 0x01, 0x00, 0x00, 0x00];

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Visit)]
struct MagicAndVersion;

encode_decode_as!(MagicAndVersion, {
    MagicAndVersion <=> MAGIC_AND_VERSION,
}, |actual| {
    Err(DecodeErrorKind::InvalidMagic { actual }.into())
});

#[derive(Wasmbin)]
#[repr(transparent)]
struct ModuleRepr {
    magic_and_version: MagicAndVersion,
    sections: Vec<Section>,
}

/// [WebAssembly Module](https://webassembly.github.io/spec/core/binary/modules.html#binary-module).
///
/// Unless you're doing something very specific, this will be your entry point to the library as it
/// represents the module as a whole. Check out its fields for nested structures.
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Module {
    /// Module [sections](https://webassembly.github.io/spec/core/binary/modules.html#sections).
    ///
    /// Note that the spec mandates a specific order in which sections must appear, but the
    /// representation here is currently a flat Vec<{enum}> for efficiency.
    ///
    /// Use [`Module::find_std_section`] and [`Module::find_std_section_mut`] to find sections
    /// of the specific type and [`Module::find_or_insert_std_section`] to insert one in the correct
    /// position.
    ///
    /// The section order will be checked both during decoding and encoding.
    pub sections: Vec<Section>,
}

impl Encode for Module {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        unsafe { &*(self as *const Module).cast::<ModuleRepr>() }.encode(w)
    }
}

impl Decode for Module {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        ModuleRepr::decode(r).map(|repr| unsafe { std::mem::transmute::<ModuleRepr, Module>(repr) })
    }
}

impl Module {
    /// Decode a module from an arbitrary input.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::BufReader;
    /// use wasmbin::Module;
    ///
    /// # fn main() -> Result<(), wasmbin::io::DecodeError> {
    /// let file = File::open("module.wasm")?;
    /// let mut reader = BufReader::new(file);
    /// let module = Module::decode_from(reader)?;
    /// println!("{module:#?}");
    /// # Ok(())
    /// # }
    /// ```
    pub fn decode_from(mut r: impl std::io::Read) -> Result<Module, DecodeError> {
        Self::decode(&mut r)
    }

    /// Encode the module into an arbitrary output.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::BufWriter;
    /// use wasmbin::Module;
    ///
    /// # fn main() -> std::io::Result<()> {
    /// let file = File::create("module.wasm")?;
    /// let mut writer = BufWriter::new(file);
    /// # let module = Module::default();
    /// module.encode_into(writer)?;
    /// # Ok(())
    /// # }
    pub fn encode_into<W: std::io::Write>(&self, mut w: W) -> std::io::Result<W> {
        self.encode(&mut w)?;
        Ok(w)
    }

    /// Find a standard section by its payload type.
    ///
    /// ## Example
    ///
    /// ```
    /// use wasmbin::{Module, sections::payload};
    ///
    /// # fn main() -> Result<(), wasmbin::io::DecodeError> {
    /// # let module = Module::default();
    /// if let Some(imports) = module.find_std_section::<payload::Import>() {
    ///    for import in imports.try_contents()? {
    ///       println!("Module imports a {:?} from {}.{}", import.desc, import.path.module, import.path.name);
    ///   }
    /// }
    /// # Ok(())
    /// # }
    pub fn find_std_section<T: StdPayload>(&self) -> Option<&Blob<T>> {
        self.sections.iter().find_map(Section::try_as)
    }

    /// Find a standard section by its payload type and return a mutable reference.
    ///
    /// ## Example
    ///
    /// ```
    /// use wasmbin::Module;
    /// use wasmbin::sections::{payload, Import, ImportPath, ImportDesc};
    ///
    /// # fn main() -> Result<(), wasmbin::io::DecodeError> {
    /// # let mut module = Module::default();
    /// if let Some(imports) = module.find_std_section_mut::<payload::Import>() {
    ///    for import in imports.try_contents_mut()? {
    ///         // Compress references to the "env" module.
    ///         if import.path.module == "env" {
    ///             import.path.module = "a".to_owned();
    ///        }
    ///   }
    /// }
    /// # Ok(())
    /// # }
    pub fn find_std_section_mut<T: StdPayload>(&mut self) -> Option<&mut Blob<T>> {
        self.sections.iter_mut().find_map(Section::try_as_mut)
    }

    /// Find a standard section by its payload type or insert it if it's not present.
    ///
    /// The section will be inserted in the correct position according to the spec and
    /// a mutable reference will be returned for further modification.
    ///
    /// ## Example
    ///
    /// ```
    /// use wasmbin::Module;
    /// use wasmbin::sections::{payload, Import, ImportPath, ImportDesc};
    /// use wasmbin::indices::TypeId;
    ///
    /// # fn main() -> Result<(), wasmbin::io::DecodeError> {
    /// # let mut module = Module::default();
    /// module
    /// .find_or_insert_std_section(|| payload::Import::default())
    /// .try_contents_mut()?
    /// .push(Import {
    ///     path: ImportPath {
    ///         module: "env".to_owned(),
    ///         name: "my_func".to_owned(),
    ///     },
    ///     desc: ImportDesc::Func(TypeId::from(42)),
    /// });
    /// # Ok(())
    /// # }
    pub fn find_or_insert_std_section<T: StdPayload>(
        &mut self,
        insert_callback: impl FnOnce() -> T,
    ) -> &mut Blob<T> {
        let mut index = self.sections.len();
        let mut insert = true;
        for (i, section) in self.sections.iter_mut().enumerate() {
            match section.kind().cmp(&T::KIND) {
                Ordering::Less => continue,
                Ordering::Equal => {
                    // We can't just `return` here due to a bug in rustc:
                    // https://github.com/rust-lang/rust/issues/70255
                    insert = false;
                }
                Ordering::Greater => {}
            }
            index = i;
            break;
        }
        if insert {
            self.sections.insert(index, insert_callback().into());
        }
        self.sections[index]
            .try_as_mut()
            .expect("internal error: couldn't convert back just inserted section")
    }
}
