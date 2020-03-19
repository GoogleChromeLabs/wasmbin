use crate::types::GlobalType;
use crate::sections::CustomSection;
use crate::builtins::Blob;
use crate::types::MemType;
use crate::instructions::Expression;
use crate::types::TableType;
use crate::types::FuncType;
use crate::sections::Func;
use std::rc::Rc;
use crate::sections::{Section, ImportPath, ImportDesc, ExportDesc};
use custom_debug::CustomDebug;
use std::iter::{FromIterator, Extend};

pub struct IndexedCollection<T> {
	items: Vec<Option<Rc<T>>>,
	deleted: Vec<usize>,
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
			items: iter.into_iter().map(|item| Some(Rc::new(item))).collect(),
			deleted: Vec::new(),
		}
	}
}

impl<T> IndexedCollection<T> {
	pub fn get(&self, index: usize) -> Option<&Rc<T>> {
		self.items.get(index)?.as_ref()
	}

	pub fn push(&mut self, value: T) -> usize {
		match self.deleted.pop() {
			Some(index) => {
				self.items[index] = Some(Rc::new(value));
				index
			}
			None => {
				let index = self.items.len();
				self.items.push(Some(Rc::new(value)));
				index
			}
		}
	}

	// Don't allow removals for referenced elements.
	// Will return a number of references as error (excluding the owner).
	pub fn try_remove(&mut self, index: usize) -> Option<Result<T, usize>> {
		let place = self.items.get_mut(index)?;
		Some(match Rc::try_unwrap(place.take()?) {
			Ok(value) => {
				self.deleted.push(index);
				Ok(value)
			}
			Err(rc) => {
				let count = Rc::strong_count(&rc) - 1;
				*place = Some(rc);
				Err(count)
			}
		})
	}

	fn get_mut_unwrap(&mut self, index: usize) -> &mut T {
		Rc::get_mut(self.items[index].as_mut().unwrap()).unwrap()
	}

	pub fn iter(&self) -> IndexedCollectionIter<'_, T> {
		self.into_iter()
	}
}

impl<T> std::ops::Index<usize> for IndexedCollection<T> {
	type Output = Rc<T>;

	fn index(&self, index: usize) -> &Rc<T> {
		self.get(index).unwrap()
	}
}

impl<T: std::fmt::Debug> std::fmt::Debug for IndexedCollection<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.debug_list().entries(self.iter()).finish()
	}
}

pub struct IndexedCollectionIter<'a, T> {
	inner: std::slice::Iter<'a, Option<Rc<T>>>,
}

impl<'a, T> Iterator for IndexedCollectionIter<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> Option<&'a T> {
		self.inner.next()?.as_deref()
	}
}

impl<'a, T> IntoIterator for &'a IndexedCollection<T> {
	type Item = &'a T;
	type IntoIter = IndexedCollectionIter<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		IndexedCollectionIter {
			inner: self.items.iter()
		}
	}
}

#[derive(Debug)]
pub enum MaybeImported<T> {
	Imported(ImportPath),
	Local(T),
}

#[derive(Debug)]
pub struct MaybeExternal<T, V> {
	pub ty: T,
	pub value: MaybeImported<V>,
	pub export_name: Option<String>,
}

#[derive(Debug)]
pub struct Element {
	pub offset: Expression,
	pub init: Vec<Rc<Function>>,
}

#[derive(CustomDebug)]
pub struct Data {
    pub offset: Expression,
    #[debug(with = "custom_debug::hexbuf_str")]
    pub init: Blob<Vec<u8>>,
}

pub type Function = MaybeExternal<Rc<FuncType>, Blob<Func>>;
pub type Table = MaybeExternal<TableType, Vec<Element>>;
pub type Memory = MaybeExternal<MemType, Vec<Data>>;
pub type Global = MaybeExternal<GlobalType, Expression>;

#[derive(Debug, Default)]
pub struct Module {
	pub types: IndexedCollection<FuncType>,
	pub functions: IndexedCollection<Function>,
	pub tables: IndexedCollection<Table>,
	pub memories: IndexedCollection<Memory>,
	pub globals: IndexedCollection<Global>,
	pub start: Option<Rc<Function>>,
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
							($dest:ident, $ty:expr) => (dest.$dest.items.push(Some(Rc::new(MaybeExternal {
								ty: $ty,
								value: MaybeImported::Imported(import.path),
								export_name: None,
							}))));
						}
						match import.desc {
							ImportDesc::Func(type_idx) => import!(functions, Rc::clone(&dest.types[type_idx.index as usize])),
							ImportDesc::Table(table_type) => import!(tables, table_type),
							ImportDesc::Mem(mem_type) => import!(memories, mem_type),
							ImportDesc::Global(global_type) => import!(globals, global_type),
						}
					}
				}
				Section::Function(Blob(type_indices)) => {
					let types = &dest.types;
					dest.functions.items.extend(type_indices.into_iter().map(|type_idx| Some(Rc::new(Function {
						ty: Rc::clone(&types[type_idx.index as usize]),
						value: MaybeImported::Local(Default::default()),
						export_name: None,
					}))));
				},
				Section::Table(Blob(table_types)) => {
					dest.tables = table_types.into_iter().map(|table_type| Table {
						ty: table_type,
						value: MaybeImported::Local(Vec::new()),
						export_name: None,
					}).collect();
				},
				Section::Memory(Blob(mem_types)) => {
					dest.memories = mem_types.into_iter().map(|mem_type| Memory {
						ty: mem_type,
						value: MaybeImported::Local(Vec::new()),
						export_name: None,
					}).collect();
				}
				Section::Global(Blob(globals)) => {
					dest.globals = globals.into_iter().map(|global| Global {
						ty: global.ty,
						value: MaybeImported::Local(global.init),
						export_name: None,
					}).collect();
				},
				Section::Export(Blob(exports)) => {
					for export in exports {
						macro_rules! export {
							($dest:ident, $idx:expr) => (dest.$dest.get_mut_unwrap($idx.index as usize).export_name = Some(export.name));
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
					dest.start = Some(Rc::clone(&dest.functions[func_idx.index as usize]));
				}
				Section::Element(Blob(elements)) => {
					// TODO: figure out if this applies to imported tables
					// let functions = &dest.functions;
					// for elem in elements {
					// 	Rc::get_mut(dest.tables.items[elem.table.index as usize].as_mut().unwrap()).unwrap().value.push(Element {
					// 		offset: elem.offset,
					// 		init: elem.init.into_iter().map(|func_idx| Rc::clone(&functions[func_idx.index as usize])).collect(),
					// 	})
					// }
				}
				Section::Code(Blob(code)) => {
					for (code, func) in code.into_iter().zip(&mut dest.functions.items) {
						Rc::get_mut(func.as_mut().unwrap()).unwrap().value = MaybeImported::Local(code);
					}
				}
				Section::Data(Blob(data)) => {
					// TODO: figure out if this applies to imported memory
				}
			}
		}
		dest
	}
}
