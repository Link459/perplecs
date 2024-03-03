use crate::{archetype::Archetype, bundle::Bundle};
use std::marker::PhantomData;

#[derive(Default)]
pub struct Query<'a, T>
where
    T: Bundle<'a>,
{
    archetypes: Box<[&'a Archetype]>,
    archetype_index: usize,
    current_index: usize,
    _phantom_data: PhantomData<T>,
}

impl<'a, T> Query<'a, T>
where
    T: Bundle<'a>,
{
    pub fn new(archetypes: Box<[&'a Archetype]>) -> Self {
        Self {
            archetypes,
            archetype_index: 0,
            current_index: 0,
            _phantom_data: PhantomData::default(),
        }
    }
}

impl<'a, T> Iterator for Query<'a, T>
where
    T: Bundle<'a> + 'a,
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
pub struct QueryMut<'a, T>
where
    T: Bundle<'a>,
{
    archetypes: Box<[&'a Archetype]>,
    archetype_index: usize,
    current_index: usize,
    _phantom_data: PhantomData<T>,
}

impl<'a, T> QueryMut<'a, T>
where
    T: Bundle<'a>,
{
    pub fn new(archetypes: Box<[&'a Archetype]>) -> Self {
        Self {
            archetypes,
            archetype_index: 0,
            current_index: 0,
            _phantom_data: PhantomData::default(),
        }
    }
}

impl<'a, T> Iterator for QueryMut<'a, T>
where
    T: Bundle<'a> + 'a,
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
