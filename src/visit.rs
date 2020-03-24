use crate::io::DecodeError;

pub enum VisitError<E> {
    LazyDecode(DecodeError),
    Custom(E),
}

impl<E: std::fmt::Display> std::fmt::Display for VisitError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VisitError::LazyDecode(err) => err.fmt(f),
            VisitError::Custom(err) => err.fmt(f),
        }
    }
}

impl<E: std::fmt::Debug> std::fmt::Debug for VisitError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VisitError::LazyDecode(err) => err.fmt(f),
            VisitError::Custom(err) => err.fmt(f),
        }
    }
}

impl<E: std::error::Error> std::error::Error for VisitError<E> {}

pub use wasmbin_derive::WasmbinVisit;
pub trait WasmbinVisit: 'static + Sized {
    fn visit<'a, T: 'static, E, F: FnMut(&'a T) -> Result<(), E>>(
        &'a self,
        mut f: F,
    ) -> Result<(), VisitError<E>> {
        self.visit_child(&mut f)
    }

    fn visit_mut<'a, T: 'static, E, F: FnMut(&'a mut T) -> Result<(), E>>(
        &'a mut self,
        mut f: F,
    ) -> Result<(), VisitError<E>> {
        self.visit_child_mut(&mut f)
    }

    fn visit_child<'a, T: 'static, E, F: FnMut(&'a T) -> Result<(), E>>(
        &'a self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        match std::any::Any::downcast_ref(self) {
            Some(v) => f(v).map_err(VisitError::Custom),
            None => self.visit_children(f),
        }
    }

    fn visit_child_mut<'a, T: 'static, E, F: FnMut(&'a mut T) -> Result<(), E>>(
        &'a mut self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        match std::any::Any::downcast_mut(self) {
            Some(v) => f(unsafe {
                // Working around an apparent bug in NLL: https://github.com/rust-lang/rust/issues/70255
                &mut *(v as *mut _)
            })
            .map_err(VisitError::Custom),
            None => self.visit_children_mut(f),
        }
    }

    fn visit_children<'a, T: 'static, E, F: FnMut(&'a T) -> Result<(), E>>(
        &'a self,
        _f: &mut F,
    ) -> Result<(), VisitError<E>> {
        Ok(())
    }

    fn visit_children_mut<'a, T: 'static, E, F: FnMut(&'a mut T) -> Result<(), E>>(
        &'a mut self,
        _f: &mut F,
    ) -> Result<(), VisitError<E>> {
        Ok(())
    }
}
