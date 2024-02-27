use rustc_hash::FxHashSet;
use std::any::TypeId;

use crate::{
    archetype::ArchetypeSet,
    bundle::Bundle,
    entity::Entity,
    query::{Query, QueryMut},
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
        self.archetypes.iter_mut().for_each(|x| x.destroy(entity));
    }

    pub fn add<'a, T>(&mut self, entity: Entity, mut data: T) -> ()
    where
        T: Bundle<'a>,
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
        unsafe {
            archetype.add(entity, &data.as_ptrs());
        };
    }

    pub fn remove<'a, T>(&mut self, entity: Entity) -> Option<()>
    where
        T: Bundle<'a>,
    {
        let type_ids = T::type_ids();
        let new_type_ids;
        let new_type_info;
        let rest;
        {
            let archetype = self.archetypes.get_by_entity_mut(entity)?;
            rest = unsafe { archetype.remove(entity, &type_ids)? };
            new_type_ids = archetype.type_ids.clone();
            new_type_info = archetype
                .type_ids
                .iter()
                .map(|x| archetype.types[x])
                .collect::<Vec<_>>();
        }

        if !self.archetypes.has(&new_type_ids) {
            self.archetypes.add(&new_type_ids, &new_type_info);
        }

        let archetype = self.archetypes.get_mut(&type_ids).unwrap();
        //whatever the hell this is
        unsafe {
            archetype.add(entity, &rest);
        };
        return Some(());
    }

    pub fn get<'a, T>(&self, entity: Entity) -> Option<T::Target>
    where
        T: 'static + Bundle<'a>,
    {
        let archetype = self.archetypes.get_by_entity(entity)?;
        let data = unsafe { archetype.get(entity, &T::type_ids())? };
        return Some(unsafe { T::from_ptr(&data) });
        //return Some(unsafe { &*(t[0] as *mut T) });
    }

    pub fn get_mut<T>(&mut self, entity: Entity) -> Option<&mut T>
    where
        T: 'static,
    {
        let archetype = self.archetypes.get_by_entity(entity)?;
        let t = unsafe { archetype.get(entity, &[TypeId::of::<T>()])? };
        return Some(unsafe { &mut *(t[0] as *mut T) });
    }

    pub fn has<'a, T>(&self, entity: Entity) -> bool
    where
        T: Bundle<'a>,
    {
        if let Some(archetype) = self.archetypes.get(&T::type_ids()) {
            return archetype.has(entity);
        }
        return false;
    }

    pub fn query<'world, 'query, T>(&'world self) -> Query<'query, T>
    where
        T: Bundle<'query>,
        'world: 'query,
    {
        let archetype = self.archetypes.get(&T::type_ids()).unwrap();
        return Query::new(archetype);
    }

    pub fn query_mut<'world, 'query, T>(&'world self) -> QueryMut<'query, T>
    where
        T: Bundle<'query>,
        'world: 'query,
    {
        //Don't unwrap
        let archetype = self.archetypes.get(&T::type_ids()).unwrap();
        return QueryMut::new(archetype);
    }
}

#[cfg(test)]
mod test {
    use core::panic;
    use std::assert_eq;

    use crate::bundle::Bundle;

    use super::World;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct TestComponent {
        a: u8,
        b: u32,
    }

    #[test]
    fn world_add() {
        type TestType = (TestComponent, u32);
        let mut w = World::new();
        for i in 0..1000 {
            let e = w.spawn();
            let test_data = (TestComponent { a: 4, b: 3 }, 3u32);
            w.add(e, test_data);
            let t = w.get::<(TestComponent,)>(e).unwrap();
            let u = w.get::<(u32,)>(e).unwrap();
            let tu = w.get::<(TestComponent, u32)>(e).unwrap();
            assert_eq!(*t, test_data.0);
            assert_eq!(*u, test_data.1);
            assert_eq!(*tu.0, test_data.0);
            assert_eq!(*tu.1, test_data.1);
        }
    }

    #[test]
    fn world_add_different() {
        let mut w = World::new();
        let e = w.spawn();
        let test_data = (TestComponent { a: 4, b: 3 }, 3u32);
        w.add(e, test_data);
        let e2 = w.spawn();
        let test_data2 = (TestComponent { a: 4, b: 3 }, 3u32, 4u64);
        w.add(e2, test_data2);

        let t = w.get::<(TestComponent,)>(e).unwrap();
        let u = w.get::<(u32,)>(e).unwrap();
        let tu = w.get::<(TestComponent, u32)>(e).unwrap();
        assert_eq!(*t, test_data.0);
        assert_eq!(*u, test_data.1);
        assert_eq!(*tu.0, test_data.0);
        assert_eq!(*tu.1, test_data.1);

        let t = w.get::<(TestComponent,)>(e2).unwrap();
        let u = w.get::<(u32,)>(e2).unwrap();
        let uw = w.get::<(u64,)>(e2).unwrap();
        let tuw = w.get::<(TestComponent, u32, u64)>(e2).unwrap();
        assert_eq!(*t, test_data2.0);
        assert_eq!(*u, test_data2.1);
        assert_eq!(*uw, test_data2.2);
        assert_eq!(*tuw.0, test_data2.0);
        assert_eq!(*tuw.1, test_data2.1);
        assert_eq!(*tuw.2, test_data2.2);
    }

    #[test]
    fn world_query() {
        type TestType = (TestComponent, u32);
        let mut w = World::new();
        let mut cmp_data = Vec::with_capacity(100);
        for i in 0..100 {
            let e = w.spawn();
            let test_data = (
                TestComponent {
                    a: i,
                    b: i as u32 + 4,
                },
                3u32,
            );
            cmp_data.push(test_data);
            w.add(e, test_data);
        }

        let q = w.query::<TestType>();
        for (j, (te, i)) in q.into_iter().enumerate() {
            let cmp = cmp_data[j];
            assert_eq!(*te, cmp.0);
            assert_eq!(*i, cmp.1);
        }
    }
}
