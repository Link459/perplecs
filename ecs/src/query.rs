use std::any::TypeId;

use crate::{archetype::Archetype, bundle::Bundle, entity::Entity};

pub struct Query<'a> {
    archetype: &'a Archetype,
    current_index: usize,
}

impl<'a> Query<'a> {
    pub fn new(archetype: &'a Archetype) -> Self {
        Self {
            archetype,
            current_index: 0,
        }
    }

    pub fn get<T>(&self) -> Option<&T>
    where
        T: Bundle,
    {
        let id = TypeId::of::<T>();
        if self.current_index >= self.archetype.len() {
            return None;
        }

        if !self.archetype.type_ids.contains(&id) {
            return None;
        }

        let ty_index = self.archetype.type_ids.binary_search(&id).ok()?;
        let data = unsafe {
            self.archetype.data[ty_index].get(&self.archetype.types[&id], self.current_index)
        };

        return Some(unsafe { &*data.cast::<T>() });
    }
}

impl<'a> Iterator for Query<'a> {
    type Item = Entity;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.archetype.len() {
            return None;
        }

        let ret = self.archetype.entities[self.current_index];
        self.current_index += 1;
        return Some(ret);
    }
}
