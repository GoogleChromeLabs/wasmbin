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

use crate::builtins::Lazy;
use crate::builtins::WasmbinCountable;
use crate::builtins::{Blob, RawBlob, String};
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
use crate::Arbitrary;
use bytes::Bytes;
use custom_debug::Debug as CustomDebug;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct ModuleNameSubSection {
    pub name: String,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct NameAssoc<I, V> {
    pub index: I,
    pub value: V,
}

impl<I, V> WasmbinCountable for NameAssoc<I, V> {}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct NameMap<I, V = String> {
    pub items: Vec<NameAssoc<I, V>>,
}

pub type IndirectNameMap<I1, I2> = NameMap<I1, NameMap<I2>>;

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum NameSubSection {
    Module(Blob<String>) = 0,
    Func(Blob<NameMap<FuncId>>) = 1,
    Local(Blob<IndirectNameMap<FuncId, LocalId>>) = 2,
    #[cfg(feature = "extended-name-section")]
    Label(Blob<IndirectNameMap<FuncId, LabelId>>) = 3,
    #[cfg(feature = "extended-name-section")]
    Type(Blob<NameMap<TypeId>>) = 4,
    #[cfg(feature = "extended-name-section")]
    Table(Blob<NameMap<TableId>>) = 5,
    #[cfg(feature = "extended-name-section")]
    Memory(Blob<NameMap<MemId>>) = 6,
    #[cfg(feature = "extended-name-section")]
    Global(Blob<NameMap<GlobalId>>) = 7,
    #[cfg(feature = "extended-name-section")]
    Elem(Blob<NameMap<ElemId>>) = 8,
    #[cfg(feature = "extended-name-section")]
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
    fn decode(r: &mut bytes::Bytes) -> Result<Self, DecodeError> {
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

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct ProducerField {
    pub name: String,
    pub values: Vec<ProducerVersionedName>,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct ProducerVersionedName {
    pub name: String,
    pub version: String,
}

#[derive(Wasmbin, CustomDebug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct RawCustomSection {
    pub name: String,
    pub data: Bytes,
}

macro_rules! define_custom_sections {
    ($($name:ident($ty:ty) = $disc:literal,)*) => {
        #[derive(Debug, Arbitrary, PartialEq, Eq, Hash, Clone)]
        pub enum CustomSection {
            $($name(Lazy<$ty>),)*
            Other(RawCustomSection),
        }

        impl CustomSection {
            pub fn name(&self) -> &str {
                match self {
                    $(Self::$name(_) => $disc,)*
                    Self::Other(raw) => &raw.name,
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
            fn decode(r: &mut bytes::Bytes) -> Result<Self, DecodeError> {
                let raw = RawCustomSection::decode(r)?;
                Ok(match raw.name.as_ref() {
                    $($disc => CustomSection::$name(Lazy::from_raw(raw.data)),)*
                    _ => CustomSection::Other(raw)
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
                    $(CustomSection::$name(data) => data.visit_child(f),)*
                    CustomSection::Other(raw) => raw.visit_child(f),
                });
                Ok(())
            }

            fn visit_children_mut<VisitT: 'static, E, F: FnMut(&mut VisitT) -> Result<(), E>>(
                &mut self,
                f: &mut F,
            ) -> Result<(), VisitError<E>> {
                // Custom section decoding errors must be ignored.
                drop(match self {
                    $(CustomSection::$name(data) => data.visit_child_mut(f),)*
                    CustomSection::Other(raw) => raw.visit_child_mut(f),
                });
                Ok(())
            }
        }
    };
}

define_custom_sections! {
    Name(Vec<NameSubSection>) = "name",
    Producers(Vec<ProducerField>) = "producers",
    // https://github.com/WebAssembly/tool-conventions/blob/08bacbed/Debugging.md#external-dwarf
    ExternalDebugInfo(String) = "external_debug_info",
    // https://github.com/WebAssembly/tool-conventions/blob/08bacbed/Debugging.md#source-maps
    SourceMappingUrl(String) = "sourceMappingURL",
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum ImportDesc {
    Func(TypeId) = 0x00,
    Table(TableType) = 0x01,
    Mem(MemType) = 0x02,
    Global(GlobalType) = 0x03,
    #[cfg(feature = "exception-handling")]
    Exception(ExceptionType) = 0x04,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct ImportPath {
    pub module: String,
    pub name: String,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Import {
    pub path: ImportPath,
    pub desc: ImportDesc,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Global {
    pub ty: GlobalType,
    pub init: Expression,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum ExportDesc {
    Func(FuncId) = 0x00,
    Table(TableId) = 0x01,
    Mem(MemId) = 0x02,
    Global(GlobalId) = 0x03,
    #[cfg(feature = "exception-handling")]
    Exception(ExceptionId) = 0x04,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum ElemKind {
    FuncRef = 0x00,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
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

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Locals {
    pub repeat: u32,
    pub ty: ValueType,
}

// https://webassembly.github.io/exception-handling/core/binary/modules.html#exception-section
#[cfg(feature = "exception-handling")]
#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[wasmbin(discriminant = 0x00)]
pub struct Exception {
    pub ty: TypeId,
}

#[derive(
    Wasmbin, WasmbinCountable, Debug, Default, Arbitrary, PartialEq, Eq, Hash, Clone, Visit,
)]
pub struct FuncBody {
    pub locals: Vec<Locals>,
    pub expr: Expression,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum DataInit {
    Active { offset: Expression } = 0x00,
    Passive = 0x01,
    ActiveWithMemory { memory: MemId, offset: Expression } = 0x02,
}

#[derive(Wasmbin, WasmbinCountable, CustomDebug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Data {
    pub init: DataInit,
    pub blob: RawBlob,
}

pub trait Payload: Encode + Decode + Into<Section> {
    const KIND: Kind;

    fn try_from_ref(section: &Section) -> Option<&Blob<Self>>;
    fn try_from_mut(section: &mut Section) -> Option<&mut Blob<Self>>;
    fn try_from(section: Section) -> Result<Blob<Self>, Section>;
}

pub trait StdPayload: Payload {}

macro_rules! define_sections {
    ($($(# $attr:tt)? $name:ident($ty:ty) = $disc:literal,)*) => {
        pub mod payload {
            $($(# $attr)? pub type $name = $ty;)*
        }

                #[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
        #[repr(u8)]
        pub enum Section {
            $($(# $attr)? $name(Blob<payload::$name>) = $disc,)*
        }

        #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
        #[repr(u8)]
        pub enum Kind {
            $($(# $attr)? $name = $disc,)*
        }

        impl TryFrom<u8> for Kind {
            type Error = u8;

            fn try_from(discriminant: u8) -> Result<Kind, u8> {
                Ok(match discriminant {
                    $($(# $attr)? $disc => Kind::$name,)*
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
                    $($(# $attr)? $name,)*
                }

                impl From<Kind> for OrderedRepr {
                    fn from(kind: Kind) -> Self {
                        match kind {
                            $($(# $attr)? Kind::$name => Self::$name,)*
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

        $($(# $attr)? const _: () = {
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
            pub fn kind(&self) -> Kind {
                match self {
                    $($(# $attr)? Section::$name(_) => Kind::$name,)*
                }
            }

            pub fn try_as<T: Payload>(&self) -> Option<&Blob<T>> {
                T::try_from_ref(self)
            }

            pub fn try_as_mut<T: Payload>(&mut self) -> Option<&mut Blob<T>> {
                T::try_from_mut(self)
            }
        }

        define_sections!(@std $($(# $attr)? $name)*);
    };

    (@std $ignore_custom:ident $($(# $attr:tt)? $name:ident)*) => {
        $($(# $attr)? impl StdPayload for payload::$name {})*
    };
}

define_sections! {
    Custom(super::CustomSection) = 0,
    Type(Vec<super::FuncType>) = 1,
    Import(Vec<super::Import>) = 2,
    Function(Vec<super::TypeId>) = 3,
    Table(Vec<super::TableType>) = 4,
    Memory(Vec<super::MemType>) = 5,
    #[cfg(feature = "exception-handling")]
    Exception(Vec<super::Exception>) = 13,
    Global(Vec<super::Global>) = 6,
    Export(Vec<super::Export>) = 7,
    Start(super::FuncId) = 8,
    Element(Vec<super::Element>) = 9,
    DataCount(u32) = 12,
    Code(Vec<super::Blob<super::FuncBody>>) = 10,
    Data(Vec<super::Data>) = 11,
}

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
    pub fn try_add(&mut self, section: &Section) -> Result<(), SectionOrderError> {
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
    fn decode(r: &mut bytes::Bytes) -> Result<Self, DecodeError> {
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
