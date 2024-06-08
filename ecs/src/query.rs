use crate::{archetype::Archetype, bundle::Bundle};
use std::{alloc::Allocator, marker::PhantomData};

pub struct Query<'a, T, A>
where
    T: Bundle<'a>,
    A: Allocator,
{
    archetypes: Box<[&'a Archetype<A>]>,
    archetype_index: usize,
    current_index: usize,
    _phantom_data: PhantomData<T>,
}

impl<'a, T, A> Query<'a, T, A>
where
    T: Bundle<'a>,
    A: Allocator,
{
    pub fn new(archetypes: Box<[&'a Archetype<A>]>) -> Self {
        Self {
            archetypes,
            archetype_index: 0,
            current_index: 0,
            _phantom_data: PhantomData::default(),
        }
    }
}

impl<'a, T, A> Iterator for Query<'a, T, A>
where
    T: Bundle<'a> + 'a,
    A: Allocator,
{
    type Item = T::Target;
    fn next(&mut self) -> Option<Self::Item> {
        let archetype = &self.archetypes[self.archetype_index];

        if self.current_index >= archetype.len() {
            self.archetype_index += 1;
            if self.archetype_index >= self.archetypes.len() {
                return None;
            }
            self.current_index = 0;
        }

        let archetype = &self.archetypes[self.archetype_index];
        let data = unsafe {
            archetype
                .get_by_index(self.current_index, &T::type_ids())
                .unwrap()
        };

        self.current_index += 1;
        return Some(unsafe { T::from_ptr(&data) });
    }
}

#[derive(Default)]
pub struct QueryMut<'a, T, A>
where
    T: Bundle<'a>,
    A: Allocator,
{
    archetypes: Box<[&'a mut Archetype<A>]>,
    archetype_index: usize,
    current_index: usize,
    _phantom_data: PhantomData<T>,
}

impl<'a, T, A> QueryMut<'a, T, A>
where
    T: Bundle<'a>,
    A: Allocator,
{
    pub fn new(archetypes: Box<[&'a mut Archetype<A>]>) -> Self {
        Self {
            archetypes,
            archetype_index: 0,
            current_index: 0,
            _phantom_data: PhantomData::default(),
        }
    }
}

impl<'a, T,A> Iterator for QueryMut<'a, T,A>
where
    T: Bundle<'a> + 'a,
    A: Allocator
{
    type Item = T::TargetMut;
    fn next(&mut self) -> Option<Self::Item> {
        let archetype = &self.archetypes[self.archetype_index];

        if self.current_index >= archetype.len() {
            self.archetype_index += 1;
            if self.archetype_index >= self.archetypes.len() {
                return None;
            }
            self.current_index = 0;
        }

        let archetype = &self.archetypes[self.archetype_index];
        let data = unsafe {
            archetype
                .get_by_index(self.current_index, &T::type_ids())
                .unwrap()
        };

        self.current_index += 1;
        return Some(unsafe { T::from_ptr_mut(&data) });
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn query_simple() {}

    #[test]
    fn query_complex() {}

    #[test]
    fn query_mut() {}
}
