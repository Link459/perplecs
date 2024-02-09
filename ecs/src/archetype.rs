use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::entity::Entity;

pub struct Archetype {
    type_ids: Vec<TypeId>,
    entities: Box<[Entity]>,
}

impl Archetype {
    pub fn new() -> Self {
        Self {
            type_ids: Vec::new(),
            entities: Box::new([]),
        }
    }

    pub fn get<'a>(entity: Entity) -> &'a dyn Any {
        todo!()
    }
    pub fn get_mut<'a>(entity: Entity) -> &'a mut dyn Any {
        todo!()
    }
}

pub struct ArchetypeSet {
    index: HashMap<Box<[TypeId]>, u32>,
    archetypes: Vec<Archetype>,
}

impl ArchetypeSet {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            archetypes: Vec::new(),
        }
    }
}
