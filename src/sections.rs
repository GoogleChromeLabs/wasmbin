//! [Module sections](https://webassembly.github.io/spec/core/binary/modules.html#sections).

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

use crate::builtins::{Blob, Lazy, UnparsedBytes, WasmbinCountable};
#[cfg(feature = "exception-handling")]
use crate::indices::ExceptionId;
#[cfg(feature = "extended-name-section")]
use crate::indices::{DataId, ElemId, LabelId};
use crate::indices::{FuncId, GlobalId, LocalId, MemId, TableId, TypeId};
use crate::instructions::Expression;
use crate::io::{Decode, DecodeError, DecodeWithDiscriminant, Encode, PathItem, Wasmbin};
#[cfg(feature = "exception-handling")]
use crate::types::ExceptionType;
use crate::types::{FuncType, GlobalType, MemType, RefType, TableType, ValueType};
use crate::visit::{Visit, VisitError};
use custom_debug::Debug as CustomDebug;
use std::convert::TryFrom;
use thiserror::Error;

/// A [name association](https://webassembly.github.io/spec/core/appendix/custom.html#binary-namemap) key-value pair.
///
/// Might also be used to represent an [indirect name association](https://webassembly.github.io/spec/core/appendix/custom.html#binary-indirectnamemap).
#[derive(Wasmbin, Debug, PartialEq, Eq, Hash, Clone, Visit)]
pub struct NameAssoc<I, V = String> {
    pub index: I,
    pub value: V,
}

impl<I, V> WasmbinCountable for NameAssoc<I, V> {}

/// [Name map](https://webassembly.github.io/spec/core/appendix/custom.html#binary-namemap).
#[derive(Wasmbin, Debug, PartialEq, Eq, Hash, Clone, Visit)]
pub struct NameMap<I, V = String> {
    pub items: Vec<NameAssoc<I, V>>,
}

/// [Indirect name map](https://webassembly.github.io/spec/core/appendix/custom.html#binary-indirectnamemap).
pub type IndirectNameMap<I1, I2> = NameMap<I1, NameMap<I2>>;

/// [Name subsection](https://webassembly.github.io/spec/core/appendix/custom.html#subsections).
#[derive(Wasmbin, Debug, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum NameSubSection {
    /// [Module name](https://webassembly.github.io/spec/core/appendix/custom.html#module-names).
    Module(Blob<String>) = 0,
    /// [Function names](https://webassembly.github.io/spec/core/appendix/custom.html#function-names).
    Func(Blob<NameMap<FuncId>>) = 1,
    /// [Local names](https://webassembly.github.io/spec/core/appendix/custom.html#local-names) grouped by function index.
    Local(Blob<IndirectNameMap<FuncId, LocalId>>) = 2,
    #[cfg(feature = "extended-name-section")]
    /// [Label names](https://www.scheidecker.net/2019-07-08-extended-name-section-spec/appendix/custom.html#label-names) grouped by function index.
    Label(Blob<IndirectNameMap<FuncId, LabelId>>) = 3,
    #[cfg(feature = "extended-name-section")]
    /// [Type names](https://www.scheidecker.net/2019-07-08-extended-name-section-spec/appendix/custom.html#type-names).
    Type(Blob<NameMap<TypeId>>) = 4,
    #[cfg(feature = "extended-name-section")]
    /// [Table names](https://www.scheidecker.net/2019-07-08-extended-name-section-spec/appendix/custom.html#table-names).
    Table(Blob<NameMap<TableId>>) = 5,
    #[cfg(feature = "extended-name-section")]
    /// [Memory names](https://www.scheidecker.net/2019-07-08-extended-name-section-spec/appendix/custom.html#memory-names).
    Memory(Blob<NameMap<MemId>>) = 6,
    #[cfg(feature = "extended-name-section")]
    /// [Global names](https://www.scheidecker.net/2019-07-08-extended-name-section-spec/appendix/custom.html#global-names).
    Global(Blob<NameMap<GlobalId>>) = 7,
    #[cfg(feature = "extended-name-section")]
    /// Element segment names.
    Elem(Blob<NameMap<ElemId>>) = 8,
    #[cfg(feature = "extended-name-section")]
    /// Data segment names.
    Data(Blob<NameMap<DataId>>) = 9,
}

impl Encode for [NameSubSection] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        for sub in self {
            sub.encode(w)?;
        }
        Ok(())
    }
}

impl Decode for Vec<NameSubSection> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut sub = Vec::new();
        while let Some(disc) = Option::decode(r)? {
            let i = sub.len();
            sub.push(
                NameSubSection::decode_with_discriminant(disc, r)
                    .map_err(move |err| err.in_path(PathItem::Index(i)))?,
            );
        }
        Ok(sub)
    }
}

/// [`producer`](https://github.com/WebAssembly/tool-conventions/blob/08bacbed7d0daff49808370cd93b6a6f0c962d76/ProducersSection.md#custom-section) field.
#[derive(Wasmbin, WasmbinCountable, Debug, PartialEq, Eq, Hash, Clone, Visit)]
pub struct ProducerField {
    pub name: String,
    pub values: Vec<ProducerVersionedName>,
}

/// [`producer`](https://github.com/WebAssembly/tool-conventions/blob/08bacbed7d0daff49808370cd93b6a6f0c962d76/ProducersSection.md#custom-section) `versioned-name` structure.
#[derive(Wasmbin, WasmbinCountable, Debug, PartialEq, Eq, Hash, Clone, Visit)]
pub struct ProducerVersionedName {
    pub name: String,
    pub version: String,
}

/// A raw [custom section](https://webassembly.github.io/spec/core/binary/modules.html#custom-section).
///
/// Used to represent custom sections with unknown semantics.
#[derive(Wasmbin, Debug, PartialEq, Eq, Hash, Clone, Visit)]
pub struct RawCustomSection {
    pub name: String,
    pub data: UnparsedBytes,
}

macro_rules! define_custom_sections {
    ($(#[doc = $url:literal] $name:ident($ty:ty) = $disc:literal,)*) => {
        /// A [custom section](https://webassembly.github.io/spec/core/binary/modules.html#custom-section).
        ///
        /// This enum supports some non-standard custom sections commonly used in tooling, but is marked
        /// as non-exhaustive to allow for future additions that would transform some sections
        /// currently represented by the [`Other`](CustomSection::Other) variant into new variants.
        #[derive(Debug, PartialEq, Eq, Hash, Clone)]
        #[non_exhaustive]
        pub enum CustomSection {
            $(
                #[doc = "[`"]
                #[doc = $disc]
                #[doc = "`]("]
                #[doc = $url]
                #[doc = ") custom section."]
                $name($ty),
            )*
            /// A custom section that is not recognized by this library.
            Other(RawCustomSection),
        }

        impl CustomSection {
            /// Name of this custom section.
            pub fn name(&self) -> &str {
                match self {
                    $(Self::$name(_) => $disc,)*
                    Self::Other(raw) => raw.name.as_str(),
                }
            }
        }

        impl Encode for CustomSection {
            fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                match self {
                    $(CustomSection::$name(data) => {
                        $disc.encode(w)?;
                        data.encode(w)
                    })*
                    CustomSection::Other(raw) => raw.encode(w)
                }
            }
        }

        impl Decode for CustomSection {
            fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
                let name = String::decode(r)?;
                Ok(match name.as_str() {
                    $($disc => CustomSection::$name(<$ty>::decode(r)?),)*
                    _ => CustomSection::Other(RawCustomSection {
                        name,
                        data: UnparsedBytes::decode(r)?
                    })
                })
            }
        }

        impl Visit for CustomSection {
            fn visit_children<'a, VisitT: 'static, E, F: FnMut(&'a VisitT) -> Result<(), E>>(
                &'a self,
                f: &mut F,
            ) -> Result<(), VisitError<E>> {
                // Custom section decoding errors must be ignored.
                drop(match self {
                    $(CustomSection::$name(data) => Visit::visit_child(data, f),)*
                    CustomSection::Other(raw) => Visit::visit_child(raw, f),
                });
                Ok(())
            }

            fn visit_children_mut<VisitT: 'static, E, F: FnMut(&mut VisitT) -> Result<(), E>>(
                &mut self,
                f: &mut F,
            ) -> Result<(), VisitError<E>> {
                // Custom section decoding errors must be ignored.
                drop(match self {
                    $(CustomSection::$name(data) => Visit::visit_child_mut(data, f),)*
                    CustomSection::Other(raw) => Visit::visit_child_mut(raw, f),
                });
                Ok(())
            }
        }
    };
}

define_custom_sections! {
    /// https://webassembly.github.io/spec/core/appendix/custom.html#name-section
    Name(Lazy<Vec<NameSubSection>>) = "name",
    /// https://github.com/WebAssembly/tool-conventions/blob/08bacbed7d0daff49808370cd93b6a6f0c962d76/ProducersSection.md
    Producers(Lazy<Vec<ProducerField>>) = "producers",
    /// https://github.com/WebAssembly/tool-conventions/blob/08bacbed/Debugging.md#external-dwarf
    ExternalDebugInfo(Lazy<String>) = "external_debug_info",
    /// https://github.com/WebAssembly/tool-conventions/blob/08bacbed/Debugging.md#source-maps
    SourceMappingUrl(Lazy<String>) = "sourceMappingURL",
    /// https://github.com/WebAssembly/tool-conventions/blob/9b80cd2339c648822bb845a083d9ffa6e20fb1ee/BuildId.md
    BuildId(Vec<u8>) = "build_id",
}

/// [Import descriptor](https://webassembly.github.io/spec/core/binary/modules.html#binary-importdesc).
#[derive(Wasmbin, Debug, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum ImportDesc {
    Func(TypeId) = 0x00,
    Table(TableType) = 0x01,
    Mem(MemType) = 0x02,
    Global(GlobalType) = 0x03,
    #[cfg(feature = "exception-handling")]
    Exception(ExceptionType) = 0x04,
}

/// [Import](https://webassembly.github.io/spec/core/binary/modules.html#import-section) path.
#[derive(Wasmbin, Debug, PartialEq, Eq, Hash, Clone, Visit)]
pub struct ImportPath {
    pub module: String,
    pub name: String,
}

/// A single [import](https://webassembly.github.io/spec/core/binary/modules.html#binary-import).
#[derive(Wasmbin, WasmbinCountable, Debug, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Import {
    pub path: ImportPath,
    pub desc: ImportDesc,
}

/// A single [global](https://webassembly.github.io/spec/core/binary/modules.html#binary-global).
#[derive(Wasmbin, WasmbinCountable, Debug, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Global {
    pub ty: GlobalType,
    pub init: Expression,
}

/// [Export descriptor](https://webassembly.github.io/spec/core/binary/modules.html#binary-exportdesc).
#[derive(Wasmbin, Debug, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum ExportDesc {
    Func(FuncId) = 0x00,
    Table(TableId) = 0x01,
    Mem(MemId) = 0x02,
    Global(GlobalId) = 0x03,
    #[cfg(feature = "exception-handling")]
    Exception(ExceptionId) = 0x04,
}

/// A single [export](https://webassembly.github.io/spec/core/binary/modules.html#binary-export).
#[derive(Wasmbin, WasmbinCountable, Debug, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}

/// [Element kind](https://webassembly.github.io/spec/core/binary/modules.html#binary-elemkind).
#[derive(Wasmbin, Debug, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum ElemKind {
    FuncRef = 0x00,
}

/// A single [element](https://webassembly.github.io/spec/core/binary/modules.html#binary-elem).
#[derive(Wasmbin, WasmbinCountable, Debug, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum Element {
    ActiveWithFuncs {
        offset: Expression,
        funcs: Vec<FuncId>,
    } = 0x00,
    PassiveWithFuncs {
        kind: ElemKind,
        funcs: Vec<FuncId>,
    } = 0x01,
    ActiveWithTableAndFuncs {
        table: TableId,
        offset: Expression,
        kind: ElemKind,
        funcs: Vec<FuncId>,
    } = 0x02,
    DeclarativeWithFuncs {
        kind: ElemKind,
        funcs: Vec<FuncId>,
    } = 0x03,
    ActiveWithExprs {
        offset: Expression,
        exprs: Vec<Expression>,
    } = 0x04,
    PassiveWithExprs {
        ty: RefType,
        exprs: Vec<Expression>,
    } = 0x05,
    ActiveWithTableAndExprs {
        table: TableId,
        offset: Expression,
        ty: RefType,
        exprs: Vec<Expression>,
    } = 0x06,
    DeclarativeWithExprs {
        ty: RefType,
        exprs: Vec<Expression>,
    } = 0x07,
}

/// Number of repeated consecutive [locals](https://webassembly.github.io/spec/core/binary/modules.html#binary-local) of a single type.
#[derive(Wasmbin, WasmbinCountable, Debug, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Locals {
    pub repeat: u32,
    pub ty: ValueType,
}

/// [Exception tag](https://webassembly.github.io/exception-handling/core/binary/modules.html#exception-section).
#[cfg(feature = "exception-handling")]
#[derive(Wasmbin, WasmbinCountable, Debug, PartialEq, Eq, Hash, Clone, Visit)]
#[wasmbin(discriminant = 0x00)]
pub struct Exception {
    pub ty: TypeId,
}

/// [Function body](https://webassembly.github.io/spec/core/binary/modules.html#binary-func).
#[derive(Wasmbin, WasmbinCountable, Debug, Default, PartialEq, Eq, Hash, Clone, Visit)]
pub struct FuncBody {
    pub locals: Vec<Locals>,
    pub expr: Expression,
}

/// [`Data`] segment initialization.
#[derive(Wasmbin, Debug, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum DataInit {
    Active { offset: Expression } = 0x00,
    Passive = 0x01,
    ActiveWithMemory { memory: MemId, offset: Expression } = 0x02,
}

/// [Data segment](https://webassembly.github.io/spec/core/binary/modules.html#binary-data).
#[derive(Wasmbin, WasmbinCountable, CustomDebug, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Data {
    pub init: DataInit,
    #[debug(with = "custom_debug::hexbuf_str")]
    pub blob: Vec<u8>,
}

mod sealed {
    use super::{Blob, Decode, Encode, Kind, Section};

    pub trait Payload: Encode + Decode + Into<Section> {
        const KIND: Kind;

        fn try_from_ref(section: &Section) -> Option<&Blob<Self>>;
        fn try_from_mut(section: &mut Section) -> Option<&mut Blob<Self>>;
        fn try_from(section: Section) -> Result<Blob<Self>, Section>;
    }
}
use sealed::Payload;

/// A common marker trait for the [standard payloads](payload).
pub trait StdPayload: Payload {}

macro_rules! define_sections {
    ($($(# $attr:tt)* $name:ident($(# $ty_attr:tt)* $ty:ty) = $disc:literal,)*) => {
        /// Payload types of the [`Section`] variants.
        pub mod payload {
            $($(# $attr)* pub type $name = $ty;)*
        }

        /// [Module section](https://webassembly.github.io/spec/core/binary/modules.html#sections).
        #[derive(Wasmbin, Debug, PartialEq, Eq, Hash, Clone, Visit)]
        #[repr(u8)]
        pub enum Section {
            $($(# $attr)* $name($(# $ty_attr)* Blob<payload::$name>) = $disc,)*
        }

        /// A kind of the [`Section`] without the payload.
        #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
        #[repr(u8)]
        pub enum Kind {
            $($(# $attr)* $name = $disc,)*
        }

        impl TryFrom<u8> for Kind {
            type Error = u8;

            fn try_from(discriminant: u8) -> Result<Kind, u8> {
                #[allow(unused_doc_comments)]
                Ok(match discriminant {
                    $($(# $attr)* $disc => Kind::$name,)*
                    _ => return Err(discriminant),
                })
            }
        }

        impl Ord for Kind {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                // Some new sections might have larger discriminants,
                // but be ordered logically between those will smaller
                // discriminants.
                //
                // To compare their Kinds in a defined order, we need an
                // intermediate enum without discriminants.
                #[derive(PartialEq, Eq, PartialOrd, Ord)]
                #[repr(u8)]
                enum OrderedRepr {
                    $($(# $attr)* $name,)*
                }

                impl From<Kind> for OrderedRepr {
                    fn from(kind: Kind) -> Self {
                        #[allow(unused_doc_comments)]
                        match kind {
                            $($(# $attr)* Kind::$name => Self::$name,)*
                        }
                    }
                }

                OrderedRepr::from(*self).cmp(&OrderedRepr::from(*other))
            }
        }

        impl PartialOrd for Kind {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        $($(# $attr)* const _: () = {
            impl From<Blob<payload::$name>> for Section {
                fn from(value: Blob<payload::$name>) -> Self {
                    Section::$name(value)
                }
            }

            impl From<payload::$name> for Section {
                fn from(value: payload::$name) -> Self {
                    Section::$name(Blob::from(value))
                }
            }

            impl Payload for payload::$name {
                const KIND: Kind = Kind::$name;

                fn try_from_ref(section: &Section) -> Option<&Blob<Self>> {
                    match section {
                        Section::$name(res) => Some(res),
                        _ => None,
                    }
                }

                fn try_from_mut(section: &mut Section) -> Option<&mut Blob<Self>> {
                    match section {
                        Section::$name(res) => Some(res),
                        _ => None,
                    }
                }

                fn try_from(section: Section) -> Result<Blob<Self>, Section> {
                    match section {
                        Section::$name(res) => Ok(res),
                        _ => Err(section),
                    }
                }
            }
        };)*

        impl Section {
            /// Get the kind of the section without its payload.
            pub fn kind(&self) -> Kind {
                #[allow(unused_doc_comments)]
                match self {
                    $($(# $attr)* Section::$name(_) => Kind::$name,)*
                }
            }

            /// Try to interpret the section as a specific payload.
            pub fn try_as<T: Payload>(&self) -> Option<&Blob<T>> {
                T::try_from_ref(self)
            }

            /// Try to interpret the section as a specific payload mutably.
            pub fn try_as_mut<T: Payload>(&mut self) -> Option<&mut Blob<T>> {
                T::try_from_mut(self)
            }
        }

        define_sections!(@std $($(# $attr)* $name)*);
    };

    (@std $(# $ignore_custom_attr:tt)* $ignore_custom:ident $($(# $attr:tt)* $name:ident)*) => {
        $($(# $attr)* impl StdPayload for payload::$name {})*
    };
}

define_sections! {
    /// [Custom section](https://webassembly.github.io/spec/core/binary/modules.html#custom-section).
    Custom(super::CustomSection) = 0,
    /// [Type section](https://webassembly.github.io/spec/core/binary/modules.html#type-section).
    Type(Vec<super::FuncType>) = 1,
    /// [Import section](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
    Import(Vec<super::Import>) = 2,
    /// [Function section](https://webassembly.github.io/spec/core/binary/modules.html#function-section).
    Function(Vec<super::TypeId>) = 3,
    /// [Table section](https://webassembly.github.io/spec/core/binary/modules.html#table-section).
    Table(Vec<super::TableType>) = 4,
    /// [Memory section](https://webassembly.github.io/spec/core/binary/modules.html#memory-section).
    Memory(Vec<super::MemType>) = 5,
    #[cfg(feature = "exception-handling")]
    /// [Exception tag section](https://webassembly.github.io/exception-handling/core/binary/modules.html#tag-section).
    Exception(Vec<super::Exception>) = 13,
    /// [Global section](https://webassembly.github.io/spec/core/binary/modules.html#global-section).
    Global(Vec<super::Global>) = 6,
    /// [Export section](https://webassembly.github.io/spec/core/binary/modules.html#export-section).
    Export(Vec<super::Export>) = 7,
    /// [Start section](https://webassembly.github.io/spec/core/binary/modules.html#start-section).
    Start(
        /// [Start function](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-start).
        super::FuncId
    ) = 8,
    /// [Element section](https://webassembly.github.io/spec/core/binary/modules.html#element-section).
    Element(Vec<super::Element>) = 9,
    /// [Data count section](https://webassembly.github.io/spec/core/binary/modules.html#binary-datacountsec).
    DataCount(
        /// Number of data segments in the [`Data`](Section::Data) section.
        u32
    ) = 12,
    /// [Code section](https://webassembly.github.io/spec/core/binary/modules.html#code-section).
    Code(Vec<super::Blob<super::FuncBody>>) = 10,
    /// [Data section](https://webassembly.github.io/spec/core/binary/modules.html#data-section).
    Data(Vec<super::Data>) = 11,
}

/// Error returned when a section is out of order.
#[derive(Debug, Error)]
#[error("Section out of order: {current:?} after {prev:?}")]
pub struct SectionOrderError {
    pub current: Kind,
    pub prev: Kind,
}

impl From<SectionOrderError> for std::io::Error {
    fn from(err: SectionOrderError) -> Self {
        Self::new(std::io::ErrorKind::InvalidData, err)
    }
}

struct SectionOrderTracker {
    last_kind: Kind,
}

impl Default for SectionOrderTracker {
    fn default() -> Self {
        Self {
            last_kind: Kind::Custom,
        }
    }
}

impl SectionOrderTracker {
    fn try_add(&mut self, section: &Section) -> Result<(), SectionOrderError> {
        match section.kind() {
            Kind::Custom => {}
            kind if kind > self.last_kind => {
                self.last_kind = kind;
            }
            kind => {
                return Err(SectionOrderError {
                    prev: self.last_kind,
                    current: kind,
                });
            }
        }
        Ok(())
    }
}

impl Encode for [Section] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        let mut section_order_tracker = SectionOrderTracker::default();
        for section in self {
            section_order_tracker.try_add(section)?;
            section.encode(w)?;
        }
        Ok(())
    }
}

impl Decode for Vec<Section> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut sections = Vec::new();
        let mut section_order_tracker = SectionOrderTracker::default();
        while let Some(disc) = Option::decode(r)? {
            let i = sections.len();
            (|| -> Result<(), DecodeError> {
                let section = Section::decode_with_discriminant(disc, r)?;
                section_order_tracker.try_add(&section)?;
                sections.push(section);
                Ok(())
            })()
            .map_err(move |err| err.in_path(PathItem::Index(i)))?;
        }
        Ok(sections)
    }
}
