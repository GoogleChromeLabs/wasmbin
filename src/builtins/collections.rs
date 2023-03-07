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

use crate::io::{Decode, DecodeError, Encode, PathItem};
use crate::visit::Visit;

pub(crate) use wasmbin_derive::WasmbinCountable;

/// A trait for types that should be count-prefixed when encoded as a list.
pub(crate) trait WasmbinCountable {}

impl<T: WasmbinCountable + Encode> Encode for [T] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.len().encode(w)?;
        for item in self {
            item.encode(w)?;
        }
        Ok(())
    }
}

impl<T> Encode for Vec<T>
where
    [T]: Encode,
{
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.as_slice().encode(w)
    }
}

impl<T, const N: usize> Encode for [T; N]
where
    [T]: Encode,
{
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.as_slice().encode(w)
    }
}

impl<T: WasmbinCountable + Decode> Decode for Vec<T> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let count = usize::decode(r)?;
        (0..count)
            .map(|i| T::decode(r).map_err(move |err| err.in_path(PathItem::Index(i))))
            .collect()
    }
}

macro_rules! impl_visit_for_iter {
    () => {
        fn visit_children<'a, VisitT: 'static, E, F: FnMut(&'a VisitT) -> Result<(), E>>(
            &'a self,
            f: &mut F,
        ) -> Result<(), crate::visit::VisitError<E>> {
            for (i, v) in self.iter().enumerate() {
                v.visit_child(f)
                    .map_err(move |err| err.in_path(crate::io::PathItem::Index(i)))?;
            }
            Ok(())
        }

        fn visit_children_mut<VisitT: 'static, E, F: FnMut(&mut VisitT) -> Result<(), E>>(
            &mut self,
            f: &mut F,
        ) -> Result<(), crate::visit::VisitError<E>> {
            for (i, v) in self.iter_mut().enumerate() {
                v.visit_child_mut(f)
                    .map_err(move |err| err.in_path(crate::io::PathItem::Index(i)))?;
            }
            Ok(())
        }
    };
}

impl<T: Visit> Visit for Vec<T> {
    impl_visit_for_iter!();
}

impl<T: Visit, const N: usize> Visit for [T; N] {
    impl_visit_for_iter!();
}

impl<T: crate::visit::Visit> crate::visit::Visit for Option<T> {
    fn visit_children<'a, VisitT: 'static, E, F: FnMut(&'a VisitT) -> Result<(), E>>(
        &'a self,
        f: &mut F,
    ) -> Result<(), crate::visit::VisitError<E>> {
        if let Some(v) = self {
            v.visit_child(f)?;
        }
        Ok(())
    }

    fn visit_children_mut<VisitT: 'static, E, F: FnMut(&mut VisitT) -> Result<(), E>>(
        &mut self,
        f: &mut F,
    ) -> Result<(), crate::visit::VisitError<E>> {
        if let Some(v) = self {
            v.visit_child_mut(f)?;
        }
        Ok(())
    }
}
