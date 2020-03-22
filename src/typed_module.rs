use crate::builtins::blob::{Blob, RawBlob};
use crate::instructions::Expression;
use crate::sections::CustomSection;
use crate::sections::Func;
use crate::sections::{ExportDesc, ImportDesc, ImportPath, Section};
use crate::types::FuncType;
use crate::types::GlobalType;
use crate::types::MemType;
use crate::types::TableType;
use crate::{DecodeError, WasmbinDecode, WasmbinEncode};
use custom_debug::CustomDebug;
use std::cell::{Ref, RefCell, RefMut};
use std::convert::TryFrom;
use std::iter::{Extend, FromIterator};
use std::rc::Rc;

type Item<T> = Rc<RefCell<T>>;

fn to_item<T>(value: T) -> Option<Item<T>> {
    Some(Rc::new(RefCell::new(value)))
}

pub struct IndexedCollection<T> {
    items: Vec<Option<Item<T>>>,
    deleted: Vec<u32>,
}

impl<T> Default for IndexedCollection<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            deleted: Vec::new(),
        }
    }
}

impl<T> FromIterator<T> for IndexedCollection<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        IndexedCollection {
            items: iter.into_iter().map(to_item).collect(),
            deleted: Vec::new(),
        }
    }
}

impl<T> IndexedCollection<T> {
    fn get_rc_refcell(&self, index: u32) -> Option<&Item<T>> {
        self.items.get(usize::try_from(index).unwrap())?.as_ref()
    }

    fn get_refcell(&self, index: u32) -> Option<&RefCell<T>> {
        self.get_rc_refcell(index).map(|r| &**r)
    }

    pub fn get(&self, index: u32) -> Option<Ref<T>> {
        self.get_refcell(index).map(RefCell::borrow)
    }

    pub fn get_mut(&mut self, index: u32) -> Option<RefMut<T>> {
        self.get_refcell(index).map(RefCell::borrow_mut)
    }

    pub fn push(&mut self, value: T) -> u32 {
        match self.deleted.pop() {
            Some(index) => {
                self.items[usize::try_from(index).unwrap()] = to_item(value);
                index
            }
            None => {
                let index = u32::try_from(self.items.len()).unwrap();
                self.items.push(to_item(value));
                index
            }
        }
    }

    // Don't allow removals for referenced elements.
    // Will return a number of references as error (excluding the owner).
    pub fn try_remove(&mut self, index: u32) -> Option<Result<T, usize>> {
        let place = self.items.get_mut(usize::try_from(index).unwrap())?;
        Some(match Rc::try_unwrap(place.take()?) {
            Ok(value) => {
                self.deleted.push(index);
                Ok(RefCell::into_inner(value))
            }
            Err(rc) => {
                let count = Rc::strong_count(&rc) - 1;
                *place = Some(rc);
                Err(count)
            }
        })
    }

    pub fn iter(&self) -> IndexedCollectionIter<'_, T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IndexedCollectionIterMut<'_, T> {
        self.into_iter()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for IndexedCollection<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

pub struct IndexedCollectionIter<'a, T> {
    inner: std::slice::Iter<'a, Option<Item<T>>>,
}

impl<'a, T> Iterator for IndexedCollectionIter<'a, T> {
    type Item = Ref<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?.as_deref()?.borrow())
    }
}

impl<'a, T> IntoIterator for &'a IndexedCollection<T> {
    type Item = Ref<'a, T>;
    type IntoIter = IndexedCollectionIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IndexedCollectionIter {
            inner: self.items.iter(),
        }
    }
}

pub struct IndexedCollectionIterMut<'a, T> {
    inner: std::slice::IterMut<'a, Option<Item<T>>>,
}

impl<'a, T> Iterator for IndexedCollectionIterMut<'a, T> {
    type Item = RefMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?.as_deref()?.borrow_mut())
    }
}

impl<'a, T> IntoIterator for &'a mut IndexedCollection<T> {
    type Item = RefMut<'a, T>;
    type IntoIter = IndexedCollectionIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IndexedCollectionIterMut {
            inner: self.items.iter_mut(),
        }
    }
}

#[derive(Debug)]
pub enum MaybeImported<T> {
    Imported(ImportPath),
    Local(T),
}

#[derive(Debug)]
pub struct Element {
    pub offset: Expression,
    pub init: Vec<Item<Function>>,
}

#[derive(CustomDebug)]
pub struct Data {
    pub offset: Expression,
    #[debug(with = "custom_debug::hexbuf_str")]
    pub init: RawBlob,
}

#[derive(Debug)]
pub struct Function {
    pub ty: Item<FuncType>,
    pub body: MaybeImported<Blob<Func>>,
    pub export_name: Option<String>,
}

#[derive(Debug)]
pub struct Global {
    pub ty: GlobalType,
    pub init: MaybeImported<Expression>,
    pub export_name: Option<String>,
}

#[derive(Debug)]
pub struct Memory {
    pub ty: MemType,
    pub import_path: Option<ImportPath>,
    pub export_name: Option<String>,
    pub init: Vec<Data>,
}

#[derive(Debug)]
pub struct Table {
    pub ty: TableType,
    pub import_path: Option<ImportPath>,
    pub export_name: Option<String>,
    pub init: Vec<Element>,
}

#[derive(Debug, Default)]
pub struct Module {
    pub types: IndexedCollection<FuncType>,
    pub functions: IndexedCollection<Function>,
    pub tables: IndexedCollection<Table>,
    pub memories: IndexedCollection<Memory>,
    pub globals: IndexedCollection<Global>,
    pub start: Option<Item<Function>>,
    pub custom: Vec<Blob<CustomSection>>,
}

impl TryFrom<super::module::Module> for Module {
    type Error = DecodeError;

    fn try_from(src: super::module::Module) -> Result<Self, DecodeError> {
        let mut dest = Self::default();
        for section in src.sections {
            match section {
                Section::Custom(custom) => {
                    dest.custom.push(custom);
                }
                Section::Type(types) => {
                    dest.types = IndexedCollection::from_iter(types.try_into_contents()?);
                }
                Section::Import(imports) => {
                    for import in imports.try_into_contents()? {
                        macro_rules! import {
                            ($dest:ident, $expr:expr) => {
                                dest.$dest.items.push(to_item($expr))
                            };
                        }
                        match import.desc {
                            ImportDesc::Func(type_id) => import!(
                                functions,
                                Function {
                                    ty: Rc::clone(
                                        dest.types.get_rc_refcell(type_id.index).unwrap()
                                    ),
                                    body: MaybeImported::Imported(import.path),
                                    export_name: None,
                                }
                            ),
                            ImportDesc::Table(ty) => import!(
                                tables,
                                Table {
                                    ty,
                                    import_path: Some(import.path),
                                    init: Vec::new(),
                                    export_name: None,
                                }
                            ),
                            ImportDesc::Mem(ty) => import!(
                                memories,
                                Memory {
                                    ty,
                                    import_path: Some(import.path),
                                    init: Vec::new(),
                                    export_name: None,
                                }
                            ),
                            ImportDesc::Global(ty) => import!(
                                globals,
                                Global {
                                    ty,
                                    init: MaybeImported::Imported(import.path),
                                    export_name: None,
                                }
                            ),
                        }
                    }
                }
                Section::Function(type_indices) => {
                    let types = &dest.types;
                    dest.functions
                        .items
                        .extend(
                            type_indices
                                .try_into_contents()?
                                .into_iter()
                                .map(|type_id| {
                                    to_item(Function {
                                        ty: Rc::clone(types.get_rc_refcell(type_id.index).unwrap()),
                                        body: MaybeImported::Local(Default::default()),
                                        export_name: None,
                                    })
                                }),
                        );
                }
                Section::Table(table_types) => {
                    dest.tables
                        .items
                        .extend(table_types.try_into_contents()?.into_iter().map(|ty| {
                            to_item(Table {
                                ty,
                                import_path: None,
                                export_name: None,
                                init: Vec::new(),
                            })
                        }));
                }
                Section::Memory(mem_types) => {
                    dest.memories
                        .items
                        .extend(mem_types.try_into_contents()?.into_iter().map(|ty| {
                            to_item(Memory {
                                ty,
                                import_path: None,
                                export_name: None,
                                init: Vec::new(),
                            })
                        }));
                }
                Section::Global(globals) => {
                    dest.globals
                        .items
                        .extend(globals.try_into_contents()?.into_iter().map(|global| {
                            to_item(Global {
                                ty: global.ty,
                                init: MaybeImported::Local(global.init),
                                export_name: None,
                            })
                        }));
                }
                Section::Export(exports) => {
                    for export in exports.try_into_contents()? {
                        macro_rules! export {
                            ($dest:ident, $idx:expr) => {
                                dest.$dest.get_mut($idx.index).unwrap().export_name =
                                    Some(export.name)
                            };
                        }
                        match export.desc {
                            ExportDesc::Func(idx) => export!(functions, idx),
                            ExportDesc::Table(idx) => export!(tables, idx),
                            ExportDesc::Mem(idx) => export!(memories, idx),
                            ExportDesc::Global(idx) => export!(globals, idx),
                        }
                    }
                }
                Section::Start(func_id) => {
                    dest.start = Some(Rc::clone(
                        dest.functions
                            .get_rc_refcell(func_id.try_into_contents()?.index)
                            .unwrap(),
                    ));
                }
                Section::Element(elements) => {
                    let functions = &dest.functions;
                    for elem in elements.try_into_contents()? {
                        dest.tables
                            .get_mut(elem.table.index)
                            .unwrap()
                            .init
                            .push(Element {
                                offset: elem.offset,
                                init: elem
                                    .init
                                    .into_iter()
                                    .map(|func_id| {
                                        Rc::clone(functions.get_rc_refcell(func_id.index).unwrap())
                                    })
                                    .collect(),
                            })
                    }
                }
                Section::Code(code) => {
                    for (code, mut func) in code
                        .try_into_contents()?
                        .into_iter()
                        .zip(&mut dest.functions)
                    {
                        func.body = MaybeImported::Local(code);
                    }
                }
                Section::Data(data) => {
                    for data in data.try_into_contents()? {
                        dest.memories
                            .get_mut(data.memory.index)
                            .unwrap()
                            .init
                            .push(Data {
                                offset: data.offset,
                                init: data.init,
                            })
                    }
                }
            }
        }
        Ok(dest)
    }
}

impl WasmbinDecode for Module {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Self::try_from(crate::module::Module::decode(r)?)
    }
}

impl WasmbinEncode for Module {
    fn encode(&self, _w: &mut impl std::io::Write) -> std::io::Result<()> {
        unimplemented!()
    }
}
