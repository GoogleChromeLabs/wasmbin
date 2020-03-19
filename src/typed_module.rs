use crate::builtins::Blob;
use crate::instructions::Expression;
use crate::sections::CustomSection;
use crate::sections::Func;
use crate::sections::{ExportDesc, ImportDesc, ImportPath, Section};
use crate::types::FuncType;
use crate::types::GlobalType;
use crate::types::MemType;
use crate::types::TableType;
use custom_debug::CustomDebug;
use std::cell::{Ref, RefCell, RefMut};
use std::convert::TryFrom;
use std::iter::{Extend, FromIterator};
use std::rc::Rc;

fn to_item<T>(value: T) -> Option<Rc<RefCell<T>>> {
    Some(Rc::new(RefCell::new(value)))
}

pub struct IndexedCollection<T> {
    items: Vec<Option<Rc<RefCell<T>>>>,
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
    fn get_rc_refcell(&self, index: u32) -> Option<&Rc<RefCell<T>>> {
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
    inner: std::slice::Iter<'a, Option<Rc<RefCell<T>>>>,
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
    inner: std::slice::IterMut<'a, Option<Rc<RefCell<T>>>>,
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
    pub init: Vec<Rc<RefCell<Function>>>,
}

#[derive(CustomDebug)]
pub struct Data {
    pub offset: Expression,
    #[debug(with = "custom_debug::hexbuf_str")]
    pub init: Blob<Vec<u8>>,
}

#[derive(Debug)]
pub struct Function {
    pub ty: Rc<RefCell<FuncType>>,
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
    pub start: Option<Rc<RefCell<Function>>>,
    pub custom: Vec<Blob<CustomSection>>,
}

impl From<super::module::Module> for Module {
    fn from(src: super::module::Module) -> Self {
        let mut dest = Self::default();
        for section in src.sections {
            match section {
                Section::Custom(custom) => {
                    dest.custom.push(custom);
                }
                Section::Type(Blob(types)) => {
                    dest.types = IndexedCollection::from_iter(types);
                }
                Section::Import(Blob(imports)) => {
                    for import in imports {
                        macro_rules! import {
                            ($dest:ident, $expr:expr) => {
                                dest.$dest.items.push(to_item($expr))
                            };
                        }
                        match import.desc {
                            ImportDesc::Func(type_idx) => import!(
                                functions,
                                Function {
                                    ty: Rc::clone(
                                        dest.types.get_rc_refcell(type_idx.index).unwrap()
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
                Section::Function(Blob(type_indices)) => {
                    let types = &dest.types;
                    dest.functions
                        .items
                        .extend(type_indices.into_iter().map(|type_idx| {
                            to_item(Function {
                                ty: Rc::clone(types.get_rc_refcell(type_idx.index).unwrap()),
                                body: MaybeImported::Local(Default::default()),
                                export_name: None,
                            })
                        }));
                }
                Section::Table(Blob(table_types)) => {
                    dest.tables.items.extend(table_types.into_iter().map(|ty| {
                        to_item(Table {
                            ty,
                            import_path: None,
                            export_name: None,
                            init: Vec::new(),
                        })
                    }));
                }
                Section::Memory(Blob(mem_types)) => {
                    dest.memories.items.extend(mem_types.into_iter().map(|ty| {
                        to_item(Memory {
                            ty,
                            import_path: None,
                            export_name: None,
                            init: Vec::new(),
                        })
                    }));
                }
                Section::Global(Blob(globals)) => {
                    dest.globals.items.extend(globals.into_iter().map(|global| {
                        to_item(Global {
                            ty: global.ty,
                            init: MaybeImported::Local(global.init),
                            export_name: None,
                        })
                    }));
                }
                Section::Export(Blob(exports)) => {
                    for export in exports {
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
                Section::Start(Blob(func_idx)) => {
                    dest.start = Some(Rc::clone(
                        dest.functions.get_rc_refcell(func_idx.index).unwrap(),
                    ));
                }
                Section::Element(Blob(elements)) => {
                    let functions = &dest.functions;
                    for elem in elements {
                        dest.tables
                            .get_mut(elem.table.index)
                            .unwrap()
                            .init
                            .push(Element {
                                offset: elem.offset,
                                init: elem
                                    .init
                                    .into_iter()
                                    .map(|func_idx| {
                                        Rc::clone(functions.get_rc_refcell(func_idx.index).unwrap())
                                    })
                                    .collect(),
                            })
                    }
                }
                Section::Code(Blob(code)) => {
                    for (code, mut func) in code.into_iter().zip(&mut dest.functions) {
                        func.body = MaybeImported::Local(code);
                    }
                }
                Section::Data(Blob(data)) => {
                    for data in data {
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
        dest
    }
}
