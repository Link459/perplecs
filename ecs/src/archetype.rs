use core::{
    alloc::{Allocator, Layout},
    any::TypeId,
    marker::PhantomData,
    ptr::{self, NonNull},
};

use alloc::{boxed::Box, vec::Vec};
use rustc_hash::FxHashMap;

use crate::entity::Entity;

#[derive(Clone, Debug)]
pub struct Archetype<A>
where
    A: Allocator,
{
    pub(crate) types: FxHashMap<TypeId, TypeInfo>,
    pub(crate) type_ids: Box<[TypeId]>,
    pub(crate) entities: Vec<Entity>, //Box<[Entity]>,
    capacity: usize,
    pub(crate) data: Box<[ComponentData<A>]>,
    pub(crate) allocator: A,
}

impl<A> Archetype<A>
where
    A: Allocator,
{
    pub fn new(type_ids: &[TypeId], type_info: &[TypeInfo], allocator: A) -> Self {
        let mut types = FxHashMap::default();
        for i in type_info {
            types.insert(i.id, *i);
        }

        let mut archetype = Self {
            types,
            type_ids: type_ids.into(),
            entities: Vec::new(),
            capacity: 16,
            data: Box::new([]),
            allocator,
        };

        let mut data = Vec::with_capacity(type_ids.len());
        for i in type_ids {
            data.push(unsafe { Self::alloc(&archetype, *i, archetype.capacity) });
        }
        archetype.data = data.into_boxed_slice();
        return archetype;
    }

    pub fn empty(&self) -> bool {
        return self.entities.is_empty();
    }

    pub fn capacity(&self) -> usize {
        return self.capacity;
    }

    pub fn len(&self) -> usize {
        return self.entities.len();
    }

    pub unsafe fn add(&mut self, entity: Entity, data: &[*mut u8]) -> () {
        if self.entities.len() >= self.capacity {
            self.grow(self.capacity * 2);
        }

        for (i, (ty, data)) in self.type_ids.iter().zip(data.iter()).enumerate() {
            let ty = &self.types[ty];
            self.data[i].set(ty, self.len(), *data);
        }

        self.entities.push(entity);
        // pushing after the data adding otherwise we would get a off by plus one
    }

    //removes the specified type_ids and returns the data for the unspecefied ones
    pub unsafe fn remove(&mut self, entity: Entity, type_id: &[TypeId]) -> Option<Box<[*mut u8]>> {
        let index = self.entities.binary_search(&entity);
        if index.is_err() {
            return None;
        }

        let index = index.unwrap();

        self.entities.remove(index);
        //remove the all the components
        let mut ret = Vec::new();
        for (data, ty_id) in self.data.iter().zip(self.type_ids.iter()) {
            let ty = self.types[ty_id];
            let last = self.capacity - 1;
            let moved = data.get(&ty, last as usize);
            if type_id.contains(ty_id) {
                unsafe {
                    let removed = data.get(&ty, index);
                    (ty.drop)(removed);
                    ptr::copy_nonoverlapping(moved, removed, ty.layout.size());
                }
                continue;
            }
            let new = self.allocator.allocate(ty.layout).unwrap().as_ptr() as *mut u8;
            //let new = alloc(ty.layout);
            ptr::copy_nonoverlapping(data.get(&ty, index), new, ty.layout.size());
            ret.push(new);
        }
        return Some(ret.into_boxed_slice());
    }

    pub unsafe fn remove_whole(&mut self, entity: Entity) -> Option<Box<[*mut u8]>> {
        //why?
        // don't do this we gonna get empty stuff
        //return self.remove(entity, &self.type_ids.clone());
        return self.remove(entity, &[]);
    }

    pub fn destroy(&mut self, entity: Entity) -> () {
        let index = self.entities.binary_search(&entity).ok();
        if index.is_none() {
            return;
        }
        let index = index.unwrap();
        self.entities.remove(index);
        //remove the all the components
        for (data, ty) in self.data.iter().zip(self.type_ids.iter()) {
            let ty = self.types[ty];
            unsafe {
                let removed = data.get(&ty, index);
                (ty.drop)(removed);
                let last = self.capacity - 1;
                let moved = data.get(&ty, last as usize);
                ptr::copy_nonoverlapping(moved, removed, ty.layout.size());
                //dealloc(removed, ty.layout);
                let removed = NonNull::new(removed).unwrap();
                self.allocator.deallocate(removed, ty.layout);
            }
        }
    }

    pub fn has(&self, entity: Entity) -> bool {
        return self.entities.contains(&entity);
    }

    pub unsafe fn get(&self, entity: Entity, type_ids: &[TypeId]) -> Option<Box<[*mut u8]>> {
        let index = self.entities.binary_search(&entity).ok()?;
        let mut res = Vec::new();
        for type_id in type_ids {
            let ty_index_matcher = || {
                for (i, t) in self.type_ids.iter().enumerate() {
                    if *t == *type_id {
                        return Some(i);
                    }
                }
                None
            };

            let ty_index = ty_index_matcher()?;

            let ty = &self.types[type_id];
            res.push(self.data[ty_index].get(ty, index));
        }
        return Some(res.into_boxed_slice());
    }

    pub unsafe fn get_by_index(&self, index: usize, type_ids: &[TypeId]) -> Option<Box<[*mut u8]>> {
        let mut res = Vec::new();
        for type_id in type_ids {
            let ty_index_matcher = || {
                for (i, t) in self.type_ids.iter().enumerate() {
                    if *t == *type_id {
                        return Some(i);
                    }
                }
                None
            };

            let ty_index = ty_index_matcher()?;

            let ty = &self.types[type_id];
            res.push(self.data[ty_index].get(ty, index));
        }
        return Some(res.into_boxed_slice());
    }

    pub unsafe fn get_by_type_index(
        &self,
        type_id: &TypeId,
        ty_index: usize,
        index: usize,
    ) -> Option<NonNull<u8>> {
        let ty = &self.types[type_id];
        return unsafe {
            Some(NonNull::new(self.data[ty_index].get(ty, index)).expect("ptr is null"))
        };
    }

    unsafe fn alloc(&self, id: TypeId, size: usize) -> ComponentData<A> {
        let info = self.types.get(&id).expect("invalid type");
        return ComponentData::new(info.layout, size, &self.allocator);
    }

    //TODO: do this correctly
    unsafe fn grow(&mut self, new_size: usize) {
        for (data, ty) in self.data.iter_mut().zip(self.type_ids.iter()) {
            let ty = self.types[ty];
            data.grow(&ty.layout, self.capacity, new_size, &self.allocator);
        }
        self.capacity = new_size;
    }

    pub fn entity_iter(&self) -> impl Iterator<Item = &Entity> {
        return self.entities.iter();
    }

    pub fn entity_iter_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        return self.entities.iter_mut();
    }
}

impl<A> Drop for Archetype<A>
where
    A: Allocator,
{
    fn drop(&mut self) {
        for (data, ty) in self.data.iter().zip(self.type_ids.iter()) {
            let ty = &self.types[ty];

            for i in 0..self.capacity {
                unsafe {
                    let ptr = data.get(ty, i);
                    (ty.drop)(ptr);
                };
            }

            let layout =
                Layout::from_size_align(ty.layout.size() * self.capacity(), ty.layout.align())
                    .unwrap();
            unsafe {
                self.allocator.deallocate(data.0, layout);
            }
            //unsafe { dealloc(data.0.as_ptr(), layout) };
        }
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct TypeInfo {
    id: TypeId,
    layout: Layout,
    array_layout: Layout,
    drop: unsafe fn(*mut u8) -> (),
}

impl TypeInfo {
    pub fn new<T: 'static>() -> Self {
        unsafe fn drop_ptr<T>(ptr: *mut u8) -> () {
            ptr.cast::<T>().drop_in_place();
        }
        Self {
            id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            array_layout: Layout::new::<T>(),
            drop: drop_ptr::<T>,
        }
    }

    pub unsafe fn drop(&self, ptr: *mut u8) -> () {
        (self.drop)(ptr)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ComponentData<A>(NonNull<u8>, PhantomData<A>);

impl<A> ComponentData<A>
where
    A: Allocator,
{
    pub unsafe fn new(layout: Layout, size: usize, allocator: &A) -> Self {
        let new_layout =
            Layout::from_size_align(layout.size() * size, layout.align()).expect("unexpected");
        let ptr = allocator
            .allocate(new_layout)
            .expect("failed to allocator")
            .as_ptr() as *mut u8;
        //let ptr = alloc(new_layout);
        let ptr = NonNull::new(ptr).expect("failed to allocate memory");
        Self::from_ptr(ptr)
    }

    pub fn from_ptr(ptr: NonNull<u8>) -> Self {
        return Self(
            ptr, //NonNull::new(ptr).expect("expected a valid pointer,got a null pointer instead"),
            PhantomData::default(),
        );
    }

    pub unsafe fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr() as *mut u8
    }

    pub unsafe fn get(&self, type_info: &TypeInfo, index: usize) -> *mut u8 {
        self.as_ptr().add(type_info.layout.size() * index)
    }

    pub unsafe fn set(&mut self, type_info: &TypeInfo, index: usize, data: *mut u8) -> () {
        let dst = self.as_ptr().add(type_info.layout.size() * index);

        ptr::copy_nonoverlapping(data, dst, type_info.layout.size());
    }

    pub unsafe fn grow(
        &mut self,
        layout: &Layout,
        old_size: usize,
        new_size: usize,
        allocator: &A,
    ) -> () {
        self.0 = {
            let old_layout = Layout::from_size_align(layout.size() * old_size, layout.align())
                .expect("way to many components");
            //let ptr = realloc(self.0.as_ptr(), old_layout, new_size * layout.size());
            let new_layout = Layout::from_size_align(new_size * layout.size(), layout.align())
                .expect("failed to grow component storage"); //let ptr = self.0.as_ptr() as NonNull<[u8]>;
                                                             //self.0.as_ptr() as [u8];
            let ptr = allocator
                .grow(self.0, old_layout, new_layout)
                .unwrap()
                .as_ptr() as *mut u8;

            NonNull::new(ptr).expect("expected a valid pointer,got a null pointer instead")
        };
    }

    pub unsafe fn as_slice(&mut self, size: usize) -> &[*mut u8] {
        let refs = &self.0.as_ptr();
        let ptr = refs as *const *mut u8;
        return core::slice::from_raw_parts(ptr, size);
    }

    pub unsafe fn dealloc(self, layout: &Layout, allocator: &A) -> () {
        //dealloc(self.0.as_ptr(), *layout);
        allocator.deallocate(self.0, layout.clone());
    }
}

pub struct ArchetypeSet<A>
where
    A: Allocator,
{
    archetypes: FxHashMap<Box<[TypeId]>, Archetype<A>>,
}

impl<A> ArchetypeSet<A>
where
    A: Allocator,
{
    pub fn new() -> Self {
        Self {
            archetypes: FxHashMap::default(),
        }
    }

    pub fn has(&self, types: &[TypeId]) -> bool {
        return self.archetypes.contains_key(types);
    }

    pub fn add(&mut self, types: &[TypeId], type_info: &[TypeInfo], allocator: A) -> () {
        self.archetypes
            .insert(types.into(), Archetype::new(types, type_info, allocator));
    }

    pub fn remove(&mut self, types: &[TypeId]) -> () {
        self.archetypes.remove(types);
    }

    pub fn get(&self, types: &[TypeId]) -> Option<&Archetype<A>> {
        return self.archetypes.get(types);
    }

    pub fn get_mut(&mut self, types: &[TypeId]) -> Option<&mut Archetype<A>> {
        return self.archetypes.get_mut(types);
    }

    pub fn get_by_entity(&self, entity: Entity) -> Option<&Archetype<A>> {
        return self.archetypes.values().filter(|x| x.has(entity)).nth(0);
    }

    pub fn get_by_entity_mut(&mut self, entity: Entity) -> Option<&mut Archetype<A>> {
        return self
            .archetypes
            .values_mut()
            .filter(|x| x.has(entity))
            .nth(0);
    }

    pub fn get_similiar(&self, types: &[TypeId]) -> Option<Box<[&Archetype<A>]>> {
        let similiar = self
            .archetypes
            .values()
            .filter(|x| x.type_ids.iter().any(|y| types.contains(y)))
            .collect::<Vec<_>>();
        return Some(similiar.into_boxed_slice());
    }

    pub fn get_similiar_mut(&mut self, types: &[TypeId]) -> Option<Box<[&mut Archetype<A>]>> {
        let similiar = self
            .archetypes
            .values_mut()
            .filter(|x| x.type_ids.iter().any(|y| types.contains(y)))
            .collect::<Vec<_>>();
        return Some(similiar.into_boxed_slice());
    }

    pub fn iter(&mut self) -> impl Iterator<Item = &Archetype<A>> {
        return self.archetypes.values();
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Archetype<A>> {
        return self.archetypes.values_mut();
    }
}

#[cfg(test)]
mod tests {
    use super::{Archetype, ComponentData, TypeInfo};
    use crate::{bundle::Bundle, entity::Entity};
    use std::{
        alloc::{dealloc, Global, Layout},
        any::TypeId,
        assert_eq,
        mem::{align_of, size_of},
        vec::Vec,
    };

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct TestComponent {
        a: u8,
        b: u32,
    }

    #[test]
    fn archetype_create() {
        let type_ids = [
            TypeId::of::<u32>(),
            TypeId::of::<f64>(),
            TypeId::of::<TestComponent>(),
        ];
        let type_infos = [
            TypeInfo::new::<u32>(),
            TypeInfo::new::<f64>(),
            TypeInfo::new::<TestComponent>(),
        ];
        let archetype = Archetype::<Global>::new(&type_ids, &type_infos, Global);
        assert_eq!(*archetype.type_ids, type_ids);
        assert!(archetype.types.contains_key(&type_ids[0]));
        assert!(archetype.types.contains_key(&type_ids[1]));
        assert!(archetype.types.contains_key(&type_ids[2]));
    }

    #[test]
    fn archetype_add() {
        let type_ids = [
            TypeId::of::<u32>(),
            TypeId::of::<u64>(),
            TypeId::of::<TestComponent>(),
        ];
        let type_infos = [
            TypeInfo::new::<u32>(),
            TypeInfo::new::<u64>(),
            TypeInfo::new::<TestComponent>(),
        ];
        let mut archetype = Archetype::<Global>::new(&type_ids, &type_infos, Global);

        let mut test_data = (3u32, 2u64, TestComponent { a: 1, b: 346 });
        let entity = Entity(3);

        //unsafe { archetype.add(entity, &test_data as *const _ as *mut u8) };
        unsafe { archetype.add(entity, &test_data.as_ptrs()) };

        let t_u32 = unsafe { *(archetype.data[0].get(&type_infos[0], 0) as *mut u32) };
        let t_u64 = unsafe { *(archetype.data[1].get(&type_infos[1], 0) as *mut u64) };
        let test_component =
            unsafe { *(archetype.data[2].get(&type_infos[2], 0) as *mut TestComponent) };

        assert_eq!(test_data.0, t_u32);
        assert_eq!(test_data.1, t_u64);
        assert_eq!(test_data.2, test_component);
        assert!(archetype.entities.contains(&entity));
    }

    #[test]
    fn archetype_remove() {
        let type_ids = [
            TypeId::of::<u32>(),
            TypeId::of::<u64>(),
            TypeId::of::<TestComponent>(),
        ];
        let type_infos = [
            TypeInfo::new::<u32>(),
            TypeInfo::new::<u64>(),
            TypeInfo::new::<TestComponent>(),
        ];
        let mut archetype = Archetype::<Global>::new(&type_ids, &type_infos, Global);

        let mut test_data = (3u32, 2u64, TestComponent { a: 1, b: 346 });
        let entity = Entity(3);

        //unsafe { archetype.add(entity, &test_data as *const _ as *mut u8) };
        unsafe { archetype.add(entity, &test_data.as_ptrs()) };

        let rest = unsafe { archetype.remove(entity, &[type_ids[0]]).unwrap() };
        assert_eq!(rest.len(), 2);
        let r1 = rest[0] as *mut u64;
        let r2 = rest[1] as *mut TestComponent;
        unsafe {
            assert_eq!(test_data.1, *r1);
            assert_eq!(test_data.2, *r2);
        }
        unsafe {
            dealloc(r1 as *mut u8, Layout::new::<u64>());
            dealloc(r2 as *mut u8, Layout::new::<TestComponent>());
        };
    }

    #[test]
    fn archetype_remove_whole() {
        let type_ids = [
            TypeId::of::<u32>(),
            TypeId::of::<u64>(),
            TypeId::of::<TestComponent>(),
        ];
        let type_infos = [
            TypeInfo::new::<u32>(),
            TypeInfo::new::<u64>(),
            TypeInfo::new::<TestComponent>(),
        ];
        let mut archetype = Archetype::<Global>::new(&type_ids, &type_infos, Global);

        let mut test_data = (3u32, 2u64, TestComponent { a: 1, b: 346 });
        let entity = Entity(3);

        //unsafe { archetype.add(entity, &test_data as *const _ as *mut u8) };
        unsafe { archetype.add(entity, &test_data.as_ptrs()) };

        let all = unsafe { archetype.remove_whole(entity).unwrap() };
        assert_eq!(all.len(), 3);
        let w = all[0] as *mut u32;
        let u = all[1] as *mut u64;
        let t = all[2] as *mut TestComponent;
        unsafe {
            assert_eq!(test_data.0, *w);
            assert_eq!(test_data.1, *u);
            assert_eq!(test_data.2, *t);
        }

        unsafe {
            dealloc(
                all[0],
                Layout::from_size_align_unchecked(size_of::<u32>(), align_of::<u32>()),
            );
            dealloc(
                all[1],
                Layout::from_size_align_unchecked(size_of::<u64>(), align_of::<u64>()),
            );
            dealloc(
                all[2],
                Layout::from_size_align_unchecked(
                    size_of::<TestComponent>(),
                    align_of::<TestComponent>(),
                ),
            );
        };
    }

    #[test]
    fn archetype_grow() {
        let type_ids = [];
        let type_infos = [];
        let mut archetype = Archetype::<Global>::new(&type_ids, &type_infos, Global);
        for _ in 0..10 {
            unsafe { archetype.grow(archetype.capacity * 2) };
        }
    }

    #[test]
    fn archetype_add_many() {
        let type_ids = [
            TypeId::of::<u32>(),
            TypeId::of::<u64>(),
            TypeId::of::<TestComponent>(),
        ];
        let type_infos = [
            TypeInfo::new::<u32>(),
            TypeInfo::new::<u64>(),
            TypeInfo::new::<TestComponent>(),
        ];
        let mut archetype = Archetype::<Global>::new(&type_ids, &type_infos, Global);

        for i in 0..12 {
            let entity = Entity(i);
            let mut test_data = (3u32, 2u64, TestComponent { a: 1, b: 346 });
            unsafe { archetype.add(entity, &test_data.as_ptrs()) };
        }
    }

    #[test]
    fn archetype_get() {}

    #[test]
    fn archetype_drop() {
        let type_ids = [
            TypeId::of::<u32>(),
            TypeId::of::<u64>(),
            TypeId::of::<TestComponent>(),
        ];
        let type_infos = [
            TypeInfo::new::<u32>(),
            TypeInfo::new::<u64>(),
            TypeInfo::new::<TestComponent>(),
        ];
        let mut archetype = Archetype::<Global>::new(&type_ids, &type_infos, Global);
        let mut data = (TestComponent { a: 3, b: 8 },);

        let entity = Entity(1);
        unsafe { archetype.add(entity, &data.as_ptrs()) };
        drop(archetype);
    }

    #[test]
    fn component_data_set() {
        let type_info = TypeInfo::new::<TestComponent>();
        let layout = type_info.layout;
        let mut data = unsafe { ComponentData::<Global>::new(layout, 5, &Global) };
        let mut test_components = Vec::new();
        for i in 0..5 {
            let component = TestComponent {
                a: i,
                b: i as u32 * 2,
            };

            test_components.push(component.clone());
            let ptr = &component as *const _ as *mut u8;
            unsafe { data.set(&type_info, i.into(), ptr) };
        }

        /*let ts = unsafe {
            Vec::<TestComponent>::from_raw_parts(data.0.as_ptr() as *mut TestComponent, 5, 16)
        };*/
        //assert_eq!(test_components, ts);
        //

        unsafe {
            let layout = Layout::from_size_align_unchecked(
                5 * size_of::<TestComponent>(),
                align_of::<TestComponent>(),
            );
            data.dealloc(&layout, &Global)
        }
    }

    #[test]
    fn component_data_get() {
        let type_info = TypeInfo::new::<TestComponent>();
        let layout = type_info.layout;
        let mut data = unsafe { ComponentData::<Global>::new(layout, 1, &Global) };
        let length = 2;
        let new_size = layout.size() * length;
        unsafe { data.grow(&layout, 1, new_size, &Global) };
        let test_component = TestComponent { a: 12, b: 634 };
        let ptr = &test_component as *const _ as *mut u8;
        unsafe { data.set(&type_info, 0, ptr) };
        let test_component2 = TestComponent { a: 32, b: 234 };
        let ptr2 = &test_component2 as *const _ as *mut u8;
        unsafe { data.set(&type_info, 1, ptr2) };

        let ptr = unsafe { data.get(&type_info, 0) as *mut TestComponent };
        let ptr = unsafe { &*ptr };
        let ptr2 = unsafe { data.get(&type_info, 1) as *mut TestComponent };
        let ptr2 = unsafe { &*ptr2 };

        assert_eq!(test_component, *ptr);
        assert_eq!(test_component2, *ptr2);

        unsafe {
            let layout = Layout::from_size_align_unchecked(
                new_size * size_of::<TestComponent>(),
                align_of::<TestComponent>(),
            );
            data.dealloc(&layout, &Global)
        }
    }

    #[test]
    fn component_data_with_small_types() {
        let type_info = TypeInfo::new::<u32>();
        let layout = type_info.layout;
        let mut data = unsafe { ComponentData::<Global>::new(layout, 1, &Global) };
        let length = 2;
        let new_size = layout.size() * length;
        unsafe { data.grow(&layout, 1, new_size, &Global) };
        let test_component = 125u32;
        let ptr = &test_component as *const _ as *mut u8;
        unsafe { data.set(&type_info, 0, ptr) };
        let test_component2 = 321u32;
        let ptr2 = &test_component2 as *const _ as *mut u8;
        unsafe { data.set(&type_info, 1, ptr2) };

        let ptr = unsafe { data.get(&type_info, 0) as *mut u32 };
        let ptr = unsafe { &*ptr };
        let ptr2 = unsafe { data.get(&type_info, 1) as *mut u32 };
        let ptr2 = unsafe { &*ptr2 };

        assert_eq!(test_component, *ptr);
        assert_eq!(test_component2, *ptr2);

        unsafe {
            let layout =
                Layout::from_size_align_unchecked(new_size * size_of::<u32>(), align_of::<u32>());
            data.dealloc(&layout, &Global)
        };
    }

    #[test]
    fn component_data_grow() {
        let type_info = TypeInfo::new::<u32>();
        let layout = type_info.layout;
        let mut data = unsafe { ComponentData::<Global>::new(layout, 1, &Global) };
        let mut old_capacity;
        let mut capacity = 1;
        for _ in 0..12 {
            old_capacity = capacity;
            capacity *= 2;
            unsafe { data.grow(&layout, old_capacity, capacity, &Global) };
        }

        unsafe {
            let layout =
                Layout::from_size_align_unchecked(capacity * size_of::<u32>(), align_of::<u32>());
            data.dealloc(&layout, &Global)
        }
    }
}
