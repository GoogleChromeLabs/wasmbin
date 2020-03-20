use once_cell::unsync::OnceCell;
use std::marker::PhantomData;
use arbitrary::Arbitrary;

#[cfg(debug_assertions)]
fn unreachable<T>() -> T {
    unreachable!()
}

#[cfg(not(debug_assertions))]
fn unreachable<T>() -> T {
    unsafe { std::hint::unreachable_unchecked() }
}

pub trait LazyTransform<I, O> {
    fn lazy_transform(input: &I) -> O;
}

pub struct LazyMut<I, O, L> {
    input: Option<I>,
    output: OnceCell<O>,
    transform: PhantomData<L>,
}

impl<I: std::fmt::Debug, O: std::fmt::Debug, L> std::fmt::Debug for LazyMut<I, O, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("LazyMut")
            .field("input", &self.input_opt())
            .field("output", &self.output_opt())
            .finish()
    }
}

impl<I: Default, O, L> Default for LazyMut<I, O, L> {
    fn default() -> Self {
        LazyMut::new(Default::default())
    }
}

impl<I: 'static, O: Arbitrary, L: 'static> Arbitrary for LazyMut<I, O, L> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        O::arbitrary(u).map(LazyMut::new_from_output)
    }

    fn arbitrary_take_rest(u: arbitrary::Unstructured) -> arbitrary::Result<Self> {
        O::arbitrary_take_rest(u).map(LazyMut::new_from_output)
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        O::size_hint(depth)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self.output.get() {
            Some(output) => Box::new(output.shrink().map(LazyMut::new_from_output)),
            None => Box::new(std::iter::empty()),
        }
    }
}

impl<I, O, L> LazyMut<I, O, L> {
    pub const fn new(input: I) -> Self {
        LazyMut {
            input: Some(input),
            output: OnceCell::new(),
            transform: PhantomData,
        }
    }

    pub fn new_from_output(output: O) -> Self {
        LazyMut {
            input: None,
            output: OnceCell::from(output),
            transform: PhantomData,
        }
    }

    pub fn input_opt(&self) -> Option<&I> {
        self.input.as_ref()
    }

    pub fn input_res(&self) -> Result<&I, &O> {
        self.input_opt()
            .ok_or_else(|| self.output_opt().unwrap_or_else(unreachable))
    }

    pub fn output_opt(&self) -> Option<&O> {
        self.output.get()
    }

    pub fn output_opt_mut(&mut self) -> Option<&mut O> {
        self.input = None;
        self.output.get_mut()
    }

    pub fn set_output(&mut self, output: O) {
        self.input = None;
        self.output = OnceCell::from(output);
    }
}

impl<I: PartialEq, O: PartialEq, L> PartialEq for LazyMut<I, O, L> {
    fn eq(&self, other: &Self) -> bool {
        self.input_res() == other.input_res()
    }
}

impl<I: Eq, O: Eq, L> Eq for LazyMut<I, O, L> {}

impl<I, O, L: LazyTransform<I, O>> LazyMut<I, O, L> {
    pub fn output(&self) -> &O {
        let input = &self.input;
        self.output
            .get_or_init(|| L::lazy_transform(input.as_ref().unwrap_or_else(unreachable)))
    }

    pub fn output_mut(&mut self) -> &mut O {
        self.output();
        self.output_opt_mut().unwrap_or_else(unreachable)
    }

    pub fn into_output(self) -> O {
        self.output();
        self.output.into_inner().unwrap_or_else(unreachable)
    }
}

impl<I, O, L: LazyTransform<I, O>> std::ops::Deref for LazyMut<I, O, L> {
    type Target = O;

    fn deref(&self) -> &O {
        self.output()
    }
}

impl<I, O, L: LazyTransform<I, O>> std::ops::DerefMut for LazyMut<I, O, L> {
    fn deref_mut(&mut self) -> &mut O {
        self.output_mut()
    }
}

impl<I, O, L> LazyMut<I, O, L> {
    pub fn try_output<E>(&self) -> Result<&O, E>
    where
        L: LazyTransform<I, Result<O, E>>,
    {
        let input = &self.input;
        self.output
            .get_or_try_init(|| L::lazy_transform(input.as_ref().unwrap_or_else(unreachable)))
    }

    pub fn try_output_mut<E>(&mut self) -> Result<&mut O, E>
    where
        L: LazyTransform<I, Result<O, E>>,
    {
        self.try_output()?;
        self.output_opt_mut().ok_or_else(unreachable)
    }

    pub fn try_into_output<E>(self) -> Result<O, E>
    where
        L: LazyTransform<I, Result<O, E>>,
    {
        self.try_output()?;
        self.output.into_inner().ok_or_else(unreachable)
    }
}
