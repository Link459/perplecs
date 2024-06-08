use core::alloc::Allocator;

use rustc_hash::FxHashSet;

use crate::{
    archetype::ArchetypeSet,
    bundle::Bundle,
    entity::Entity,
    query::{Query, QueryMut},
};

pub struct World<A>
where
    A: Allocator,
{
    archetypes: ArchetypeSet<A>,
    entities: FxHashSet<Entity>,
    next_entity: u64,
}

impl<A> World<A>
where
    A: Allocator,
{
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
        //unoptimal but it works
        let type_ids = T::type_ids();
        let type_infos = T::type_info();
        let mut new_data = unsafe { data.as_ptrs() };

        let archetype = match self.archetypes.get_mut(&type_ids) {
            Some(a) => a,
            None => {
                let archetype = self.archetypes.get_by_entity_mut(entity);
                if archetype.is_none() {
                    if !self.archetypes.has(&type_ids) {
                        self.archetypes.add(&type_ids, &type_infos);

                        let archetype = self.archetypes.get_mut(&type_ids).unwrap();

                        unsafe { archetype.add(entity, &new_data) };
                    }
                    return;
                }
                let archetype = archetype.unwrap();

                let old_data = unsafe {
                    let Some(old_data) = archetype.remove_whole(entity) else {
                        return;
                    };
                    old_data
                };

                new_data = [old_data, unsafe { data.as_ptrs() }]
                    .concat()
                    .into_boxed_slice();
                let new_type_ids = [archetype.type_ids.as_ref(), type_ids.as_ref()].concat();
                let new_type_infos = [
                    archetype
                        .types
                        .clone()
                        .into_values()
                        .collect::<Vec<_>>()
                        .as_slice(),
                    type_infos.as_ref(),
                ]
                .concat();

                self.archetypes.add(&new_type_ids, &new_type_infos);
                let new_archetype = self.archetypes.get_mut(&new_type_ids).unwrap();
                new_archetype
            }
        };

        unsafe {
            //archetype.add(entity, &data.as_ptrs());
            archetype.add(entity, &new_data);
        };
    }

    pub fn remove<'a, T>(&mut self, entity: Entity) -> Option<()>
    where
        T: Bundle<'a>,
    {
        let type_ids = T::type_ids();
        let rest;

        let archetype = self.archetypes.get_by_entity_mut(entity)?;
        rest = unsafe { archetype.remove(entity, &type_ids)? };
        let new_type_ids = archetype.type_ids.clone();
        let new_type_info = archetype
            .type_ids
            .iter()
            .map(|x| archetype.types[x])
            .collect::<Vec<_>>();

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

    pub fn get_mut<'a, T>(&mut self, entity: Entity) -> Option<T::TargetMut>
    where
        T: 'static + Bundle<'a>,
    {
        let archetype = self.archetypes.get_by_entity(entity)?;
        let data = unsafe { archetype.get(entity, &T::type_ids())? };
        return Some(unsafe { T::from_ptr_mut(&data) });
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

    pub fn query<'world, 'query, T>(&'world self) -> Query<'query, T, A>
    where
        T: Bundle<'query>,
        'world: 'query,
    {
        let archetype = self
            .archetypes
            .get_similiar(&T::type_ids())
            .unwrap_or_default();
        return Query::new(archetype);
    }

    pub fn query_mut<'world, 'query, T>(&'world mut self) -> QueryMut<'query, T, A>
    where
        T: Bundle<'query>,
        'world: 'query,
    {
        //Don't unwrap
        let archetype = self
            .archetypes
            .get_similiar_mut(&T::type_ids())
            .unwrap_or_default();
        return QueryMut::new(archetype);
    }
}

#[cfg(test)]
mod test {
    use std::alloc::Global;
    use std::assert_eq;
    use std::collections::HashSet;

    type World = super::World<Global>;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct TestComponent {
        a: u8,
        b: u32,
    }

    #[test]
    fn world_add() {
        let mut w = World::new();
        for _ in 0..100 {
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
    fn world_add_after() {
        let mut w = World::new();
        let e = w.spawn();
        let test_data = (TestComponent { a: 4, b: 3 }, 3u32);
        w.add(e, (test_data.0,));
        w.add(e, (test_data.1,));

        let t = w.get::<(TestComponent,)>(e).unwrap();
        let u = w.get::<(u32,)>(e).unwrap();
        let tu = w.get::<(TestComponent, u32)>(e).unwrap();
        assert_eq!(*t, test_data.0);
        assert_eq!(*u, test_data.1);
        assert_eq!(*tu.0, test_data.0);
        assert_eq!(*tu.1, test_data.1);
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

    #[test]
    fn world_query_mut() {
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

        let q = w.query_mut::<TestType>();
        for (te, i) in q {
            *i += 1;
            te.a += 3;
            te.b += te.a as u32;
        }

        let q = w.query::<TestType>();
        for (j, (te, i)) in q.into_iter().enumerate() {
            let cmp = cmp_data[j];
            let te_cmp = TestComponent {
                a: cmp.0.a + 3,
                b: cmp.0.b + cmp.0.a as u32 + 3,
            };
            assert_eq!(*te, te_cmp);
            assert_eq!(*i, cmp.1 + 1);
        }
    }

    #[test]
    fn world_query_multi_archetype() {
        let mut w = World::new();
        let mut cmp_data = HashSet::with_capacity(20);
        for i in 0..10 {
            let e = w.spawn();
            let test_data = (
                TestComponent {
                    a: i,
                    b: i as u32 + 4,
                },
                3u32,
            );
            cmp_data.insert(test_data.0);
            w.add(e, test_data);
        }

        for i in 0..10 {
            let e = w.spawn();
            let test_data = (TestComponent { a: i, b: i as u32 },);
            cmp_data.insert(test_data.0);
            w.add(e, test_data);
        }

        let mut count = 0;
        for te in w.query::<(TestComponent,)>() {
            assert!(cmp_data.contains(te));
            count += 1;
        }

        assert_eq!(count, 20);
    }
}
