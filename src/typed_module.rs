use crate::builtins::Blob;
use crate::indices::{FuncId, GlobalId, LocalId, MemId, TableId, TypeId};
use crate::instructions::Expression;
use crate::io::{Decode, DecodeError, Encode};
use crate::sections::{
    CustomSection, DataInit, ElementInit, ExportDesc, ImportDesc, ImportPath, NameSubSection,
    RawCustomSection, Section,
};
use crate::types::{FuncType, GlobalType, MemType, TableType, ValueType};
use std::convert::TryFrom;
use std::iter::{Extend, FromIterator};
use std::marker::PhantomData;

pub struct Map<I, T> {
    items: Vec<Option<T>>,
    id_marker: PhantomData<fn() -> I>,
}

impl<I, T> Default for Map<I, T> {
    fn default() -> Self {
        Self {
            items: Vec::default(),
            id_marker: PhantomData,
        }
    }
}

impl<I, T> Extend<T> for Map<I, T> {
    fn extend<Iter: IntoIterator<Item = T>>(&mut self, iter: Iter) {
        self.items.extend(iter.into_iter().map(Some))
    }
}

impl<I, T> FromIterator<T> for Map<I, T> {
    fn from_iter<Iter: IntoIterator<Item = T>>(iter: Iter) -> Self {
        Map {
            items: iter.into_iter().map(Some).collect(),
            id_marker: PhantomData,
        }
    }
}

impl<I: Into<u32>, T> Map<I, T> {
    pub fn get(&self, id: I) -> Option<&T> {
        self.items.get(id.into() as usize)?.as_ref()
    }

    pub fn get_mut(&mut self, id: I) -> Option<&mut T> {
        self.items.get_mut(id.into() as usize)?.as_mut()
    }

    pub fn remove(&mut self, id: I) -> Option<T> {
        self.items.remove(id.into() as usize)
    }
}

impl<I: From<u32>, T> Map<I, T> {
    pub fn push(&mut self, value: T) -> I {
        let index = u32::try_from(self.items.len()).unwrap().into();
        self.items.push(Some(value));
        index
    }

    pub fn iter(&self) -> MapIter<I, T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> MapIterMut<I, T> {
        self.into_iter()
    }

    pub fn values(&self) -> MapValuesIter<T> {
        self.items.iter().into()
    }

    pub fn values_mut(&mut self) -> MapValuesIterMut<T> {
        self.items.iter_mut().into()
    }
}

impl<I: From<u32> + Into<u32>, T> std::ops::Index<I> for Map<I, T> {
    type Output = T;

    fn index(&self, id: I) -> &T {
        self.items[id.into() as usize]
            .as_ref()
            .expect("item at requested index was deleted")
    }
}

impl<I: From<u32> + Into<u32>, T> std::ops::IndexMut<I> for Map<I, T> {
    fn index_mut(&mut self, id: I) -> &mut T {
        self.items[id.into() as usize]
            .as_mut()
            .expect("item at requested index was deleted")
    }
}

impl<I: From<u32> + std::fmt::Debug, T: std::fmt::Debug> std::fmt::Debug for Map<I, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

pub struct ConvertValue<T, Iter> {
    inner: Iter,
    marker: PhantomData<fn(Iter) -> T>,
}

impl<T, Iter> From<Iter> for ConvertValue<T, Iter> {
    fn from(iter: Iter) -> Self {
        Self {
            inner: iter,
            marker: PhantomData,
        }
    }
}

impl<T, Iter: Iterator> Iterator for ConvertValue<T, Iter>
where
    Iter::Item: Into<Option<T>>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()?.into()
    }
}

pub struct ConvertIndex<I, Iter> {
    inner: std::iter::Enumerate<Iter>,
    marker: PhantomData<fn() -> I>,
}

impl<I, Iter: Iterator> From<Iter> for ConvertIndex<I, Iter> {
    fn from(iter: Iter) -> Self {
        Self {
            inner: iter.enumerate(),
            marker: PhantomData,
        }
    }
}

impl<I: From<u32>, Iter: Iterator> Iterator for ConvertIndex<I, Iter> {
    type Item = (I, Iter::Item);

    fn next(&mut self) -> Option<(I, Iter::Item)> {
        let (index, value) = self.inner.next()?;
        Some((u32::try_from(index).unwrap().into(), value))
    }
}

pub type ConvertKV<I, T, Iter> = ConvertIndex<I, ConvertValue<T, Iter>>;

impl<I: From<u32>, T, Iter: Iterator> From<Iter> for ConvertKV<I, T, Iter>
where
    Iter::Item: Into<Option<T>>,
{
    fn from(iter: Iter) -> Self {
        ConvertIndex::from(ConvertValue::from(iter))
    }
}

pub type MapValuesIter<'a, T> = ConvertValue<&'a T, std::slice::Iter<'a, Option<T>>>;

pub type MapIter<'a, I, T> = ConvertKV<I, &'a T, std::slice::Iter<'a, Option<T>>>;

impl<'a, I: From<u32>, T> IntoIterator for &'a Map<I, T> {
    type Item = (I, &'a T);
    type IntoIter = MapIter<'a, I, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter().into()
    }
}

pub type MapValuesIterMut<'a, T> = ConvertValue<&'a mut T, std::slice::IterMut<'a, Option<T>>>;

pub type MapIterMut<'a, I, T> = ConvertKV<I, &'a mut T, std::slice::IterMut<'a, Option<T>>>;

impl<'a, I: From<u32>, T> IntoIterator for &'a mut Map<I, T> {
    type Item = (I, &'a mut T);
    type IntoIter = MapIterMut<'a, I, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter_mut().into()
    }
}

pub type MapIntoIter<I, T> = ConvertKV<I, T, std::vec::IntoIter<Option<T>>>;

impl<I: From<u32>, T> IntoIterator for Map<I, T> {
    type Item = (I, T);
    type IntoIter = MapIntoIter<I, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter().into()
    }
}

#[derive(Debug)]
pub enum MaybeImported<T> {
    Imported(ImportPath),
    Local(T),
}

#[derive(Debug)]
pub struct Local {
    pub name: Option<String>,
    pub ty: ValueType,
}

#[derive(Default, Debug)]
pub struct FuncBody {
    pub locals: Map<LocalId, Local>,
    pub expr: Expression,
}

impl From<crate::sections::FuncBody> for FuncBody {
    fn from(body: crate::sections::FuncBody) -> Self {
        Self {
            locals: body
                .locals
                .into_iter()
                .flat_map(|locals| {
                    std::iter::repeat(locals.ty)
                        .map(|ty| Local { name: None, ty })
                        .take(locals.repeat as usize)
                })
                .collect(),
            expr: body.expr,
        }
    }
}

#[derive(Debug)]
pub struct Function {
    pub ty: TypeId,
    pub body: MaybeImported<FuncBody>,
    pub name: Option<String>,
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
    pub init: Vec<DataInit>,
}

#[derive(Debug)]
pub struct Table {
    pub ty: TableType,
    pub import_path: Option<ImportPath>,
    pub export_name: Option<String>,
    pub init: Vec<ElementInit>,
}

#[derive(Debug, Default)]
pub struct Module {
    pub name: Option<String>,
    pub types: Map<TypeId, FuncType>,
    pub functions: Map<FuncId, Function>,
    pub tables: Map<TableId, Table>,
    pub memories: Map<MemId, Memory>,
    pub globals: Map<GlobalId, Global>,
    pub start: Option<Blob<FuncId>>,
    pub custom: Vec<RawCustomSection>,
}

impl TryFrom<super::module::Module> for Module {
    type Error = DecodeError;

    #[allow(clippy::too_many_lines)]
    fn try_from(src: super::module::Module) -> Result<Self, DecodeError> {
        let mut dest = Self::default();
        let mut import_func_count = 0;
        for section in src.sections {
            match section {
                Section::Type(types) => {
                    dest.types = Map::from_iter(types.try_into_contents()?);
                }
                Section::Import(imports) => {
                    for import in imports.try_into_contents()? {
                        match import.desc {
                            ImportDesc::Func(ty) => {
                                dest.functions.push(Function {
                                    ty,
                                    body: MaybeImported::Imported(import.path),
                                    name: None,
                                    export_name: None,
                                });
                            }
                            ImportDesc::Table(ty) => {
                                dest.tables.push(Table {
                                    ty,
                                    import_path: Some(import.path),
                                    init: Vec::new(),
                                    export_name: None,
                                });
                            }
                            ImportDesc::Mem(ty) => {
                                dest.memories.push(Memory {
                                    ty,
                                    import_path: Some(import.path),
                                    init: Vec::new(),
                                    export_name: None,
                                });
                            }
                            ImportDesc::Global(ty) => {
                                dest.globals.push(Global {
                                    ty,
                                    init: MaybeImported::Imported(import.path),
                                    export_name: None,
                                });
                            }
                        }
                    }
                    import_func_count = dest.functions.items.len();
                }
                Section::Function(type_indices) => {
                    dest.functions
                        .extend(
                            type_indices
                                .try_into_contents()?
                                .into_iter()
                                .map(|ty| Function {
                                    ty,
                                    body: MaybeImported::Local(FuncBody::default()),
                                    name: None,
                                    export_name: None,
                                }),
                        );
                }
                Section::Table(table_types) => {
                    dest.tables
                        .extend(
                            table_types
                                .try_into_contents()?
                                .into_iter()
                                .map(|ty| Table {
                                    ty,
                                    import_path: None,
                                    export_name: None,
                                    init: Vec::new(),
                                }),
                        );
                }
                Section::Memory(mem_types) => {
                    dest.memories
                        .extend(mem_types.try_into_contents()?.into_iter().map(|ty| Memory {
                            ty,
                            import_path: None,
                            export_name: None,
                            init: Vec::new(),
                        }));
                }
                Section::Global(globals) => {
                    dest.globals
                        .extend(
                            globals
                                .try_into_contents()?
                                .into_iter()
                                .map(|global| Global {
                                    ty: global.ty,
                                    init: MaybeImported::Local(global.init),
                                    export_name: None,
                                }),
                        );
                }
                Section::Export(exports) => {
                    for export in exports.try_into_contents()? {
                        let export_name = Some(export.name);
                        match export.desc {
                            ExportDesc::Func(id) => dest.functions[id].export_name = export_name,
                            ExportDesc::Table(id) => dest.tables[id].export_name = export_name,
                            ExportDesc::Mem(id) => dest.memories[id].export_name = export_name,
                            ExportDesc::Global(id) => dest.globals[id].export_name = export_name,
                        };
                    }
                }
                Section::Start(func) => {
                    dest.start = Some(func);
                }
                Section::Element(elements) => {
                    for elem in elements.try_into_contents()? {
                        dest.tables[elem.table].init.push(elem.init);
                    }
                }
                Section::Code(code) => {
                    for (code, func) in code
                        .try_into_contents()?
                        .into_iter()
                        .zip(dest.functions.values_mut().skip(import_func_count))
                    {
                        func.body = MaybeImported::Local(code.try_into_contents()?.into());
                    }
                }
                Section::Data(data) => {
                    for data in data.try_into_contents()? {
                        dest.memories[data.memory].init.push(data.init);
                    }
                }
                Section::Custom(custom) => {
                    // TODO: technically custom sections are not supposed to error out.
                    // Decide what should we do here instead.
                    match custom.try_into_contents()? {
                        CustomSection::Name(names) => {
                            for name in names.try_into_contents()? {
                                match name {
                                    NameSubSection::Module(name) => {
                                        dest.name = Some(name.try_into_contents()?);
                                    }
                                    NameSubSection::Func(names) => {
                                        for assoc in names.try_into_contents()?.items {
                                            dest.functions[assoc.index].name = Some(assoc.value);
                                        }
                                    }
                                    NameSubSection::Local(names) => {
                                        for func_assoc in names.try_into_contents()?.items {
                                            let locals = match &mut dest.functions[func_assoc.index].body {
                                                MaybeImported::Local(body) => &mut body.locals,
                                                // TODO: either ignore this or turn into Result error.
                                                _ => panic!("Tried to set local names of an imported function."),
                                            };
                                            for local_assoc in func_assoc.value.items {
                                                locals[local_assoc.index].name =
                                                    Some(local_assoc.value);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        CustomSection::Other(raw) => {
                            dest.custom.push(raw);
                        }
                    }
                }
            }
        }
        Ok(dest)
    }
}

impl Decode for Module {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Self::try_from(crate::module::Module::decode(r)?)
    }
}

impl Encode for Module {
    fn encode(&self, _w: &mut impl std::io::Write) -> std::io::Result<()> {
        unimplemented!()
    }
}
