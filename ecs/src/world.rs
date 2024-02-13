use rustc_hash::FxHashSet;
use std::any::{Any, TypeId};

use crate::{
    archetype::{ArchetypeSet, TypeInfo},
    bundle::Bundle,
    entity::Entity,
};

pub struct World {
    archetypes: ArchetypeSet,
    entities: FxHashSet<Entity>,
    next_entity: u64,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetypes: ArchetypeSet::new(),
            entities: FxHashSet::default(),
            next_entity: 0,
        }
    }

    pub fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_entity);
        self.entities.insert(entity);
        self.next_entity += 1;
        return entity;
    }

    pub fn destroy(&mut self, entity: Entity) -> () {
        self.entities.remove(&entity);
        self.archetypes.iter_mut().for_each(|x| x.remove(entity));
    }

    pub fn add<T>(&mut self, entity: Entity, data: T) -> ()
    where
        T: Bundle,
    {
        //let id = TypeId::of::<T>();
        //let types = [id];
        let type_ids = T::type_ids();
        let type_infos = T::type_info();
        if !self.archetypes.has(&type_ids) {
            self.archetypes.add(&type_ids, &type_infos);
        }

        let archetype = self.archetypes.get_mut(&type_ids).unwrap();
        //whatever the hell this is
        unsafe { archetype.add(entity, (&data as *const T) as *mut u8) };
    }

    pub fn remove<T>(&mut self, entity: Entity) -> T
    where
        T: Bundle,
    {
        
        todo!()
    }
}
