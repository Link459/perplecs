use std::{any::TypeId, marker::PhantomData};

use crate::{archetype::Archetype, bundle::Bundle, entity::Entity};

pub struct Query<'a, T>
where
    T: Bundle,
{
    archetype: &'a Archetype,
    current_index: usize,
    _phantom_data: PhantomData<T>,
}

impl<'a, T> Query<'a, T>
where
    T: Bundle,
{
    pub fn new(archetype: &'a Archetype) -> Self {
        Self {
            archetype,
            current_index: 0,
            _phantom_data: PhantomData::default(),
        }
    }
}

impl<'a, T> Iterator for Query<'a, T>
where
    T: Bundle,
{
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.archetype.len() {
            return None;
        }

        /*let ret = self.archetype.
        self.current_index += 1;
        return Some(ret);*/
        None
    }
}

pub struct QueryMut<'a, T>
where
    T: Bundle,
{
    archetype: &'a Archetype,
    current_index: usize,
    _phantom_data: PhantomData<T>,
}

impl<'a, T> QueryMut<'a, T>
where
    T: Bundle,
{
    pub fn new(archetype: &'a Archetype) -> Self {
        Self {
            archetype,
            current_index: 0,
            _phantom_data: PhantomData::default(),
        }
    }
}

impl<'a, T> Iterator for QueryMut<'a, T>
where
    T: Bundle,
{
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.archetype.len() {
            return None;
        }

        /*let ret = self.archetype.
        self.current_index += 1;
        return Some(ret);*/
        None
    }
}
