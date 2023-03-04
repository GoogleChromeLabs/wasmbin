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

use crate::io::{DecodeError, PathItem};
use std::convert::Infallible;
use thiserror::Error;

pub use wasmbin_derive::Visit;

#[derive(Error, Debug)]
pub enum VisitError<E> {
    #[error(transparent)]
    LazyDecode(DecodeError),

    #[error(transparent)]
    Custom(E),
}

impl<E> VisitError<E> {
    pub(crate) fn in_path(self, item: PathItem) -> Self {
        #[allow(clippy::match_wildcard_for_single_variants)]
        match self {
            VisitError::LazyDecode(err) => VisitError::LazyDecode(err.in_path(item)),
            err => err,
        }
    }
}

impl From<VisitError<Infallible>> for DecodeError {
    fn from(err: VisitError<Infallible>) -> Self {
        match err {
            VisitError::Custom(err) => match err {},
            VisitError::LazyDecode(err) => err,
        }
    }
}

pub trait VisitResult {
    type Error;

    fn into_result(self) -> Result<(), Self::Error>;
}

impl VisitResult for () {
    type Error = Infallible;

    fn into_result(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl VisitResult for bool {
    type Error = ();

    fn into_result(self) -> Result<(), Self::Error> {
        match self {
            true => Ok(()),
            false => Err(()),
        }
    }
}

impl<E> VisitResult for Result<(), E> {
    type Error = E;

    fn into_result(self) -> Result<(), Self::Error> {
        self
    }
}

pub trait Visit: 'static + Sized {
    fn visit<'a, T: 'static, R: VisitResult, F: FnMut(&'a T) -> R>(
        &'a self,
        mut f: F,
    ) -> Result<(), VisitError<R::Error>> {
        self.visit_child(&mut move |item| f(item).into_result())
    }

    fn visit_mut<T: 'static, R: VisitResult, F: FnMut(&mut T) -> R>(
        &mut self,
        mut f: F,
    ) -> Result<(), VisitError<R::Error>> {
        self.visit_child_mut(&mut move |item| f(item).into_result())
    }

    fn visit_child<'a, T: 'static, E, F: FnMut(&'a T) -> Result<(), E>>(
        &'a self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        if let Some(v) = <dyn std::any::Any>::downcast_ref(self) {
            f(v).map_err(VisitError::Custom)?;
        }
        self.visit_children(f)
    }

    fn visit_child_mut<T: 'static, E, F: FnMut(&mut T) -> Result<(), E>>(
        &mut self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        if let Some(v) = <dyn std::any::Any>::downcast_mut(self) {
            f(v).map_err(VisitError::Custom)?;
        }
        self.visit_children_mut(f)
    }

    fn visit_children<'a, T: 'static, E, F: FnMut(&'a T) -> Result<(), E>>(
        &'a self,
        _f: &mut F,
    ) -> Result<(), VisitError<E>> {
        Ok(())
    }

    fn visit_children_mut<T: 'static, E, F: FnMut(&mut T) -> Result<(), E>>(
        &mut self,
        _f: &mut F,
    ) -> Result<(), VisitError<E>> {
        Ok(())
    }
}
