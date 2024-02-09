use std::any::Any;

use crate::{archetype::ArchetypeSet, entity::Entity};

pub struct World {
    archetypes: ArchetypeSet,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetypes: ArchetypeSet::new(),
        }
    }

    pub fn spawn() -> Entity {
        todo!()
    }
    pub fn add<T>() -> ()
    where
        T: Any,
    {
        todo!()
    }

    pub fn remove<T>() -> T
    where
        T: Any,
    {
        todo!()
    }
}
