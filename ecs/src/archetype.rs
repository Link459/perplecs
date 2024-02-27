use std::{
    alloc::{dealloc, realloc, Layout},
    any::TypeId,
    ptr::{self, NonNull},
};

use rustc_hash::FxHashMap;
use std::alloc::alloc;

use crate::entity::Entity;

#[derive(Default, Clone, Debug)]
pub struct Archetype {
    pub(crate) types: FxHashMap<TypeId, TypeInfo>,
    pub(crate) type_ids: Box<[TypeId]>,
    pub(crate) entities: Vec<Entity>, //Box<[Entity]>,
    capacity: usize,
    pub(crate) data: Box<[ComponentData]>,
}

impl Archetype {
    pub fn new(type_ids: &[TypeId], type_info: &[TypeInfo]) -> Self {
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
            let new = alloc(ty.layout);
            ptr::copy_nonoverlapping(data.get(&ty, index), new, ty.layout.size());
            ret.push(new);
        }
        return Some(ret.into_boxed_slice());
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

    pub unsafe fn get_by_index(
        &self,
        index: usize,
        type_ids: &[TypeId],
    ) -> Option<Box<[*mut u8]>> {
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

    unsafe fn alloc(&self, id: TypeId, size: usize) -> ComponentData {
        let info = self.types.get(&id).expect("invalid type");
        return ComponentData::new(info.layout, size);
    }

    //TODO: do this correctly
    unsafe fn grow(&mut self, new_size: usize) {
        for (data, ty) in self.data.iter_mut().zip(self.type_ids.iter()) {
            let ty = self.types[ty];
            data.grow(&ty.layout, self.capacity, new_size);
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

impl Drop for Archetype {
    fn drop(&mut self) {
        for (data, ty) in self.data.iter().zip(self.type_ids.iter()) {
            let ty = &self.types[ty];

            for i in 0..self.capacity {
                unsafe {
                    let ptr = data.get(ty, i);
                    (ty.drop)(ptr);
                };
            }

            unsafe { dealloc(data.0.as_ptr(), ty.layout) };
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
pub struct ComponentData(NonNull<u8>);

impl ComponentData {
    pub unsafe fn new(layout: Layout, size: usize) -> Self {
        let new_layout =
            Layout::from_size_align(layout.size() * size, layout.align()).expect("unexpected");
        let ptr = alloc(new_layout);
        Self::from_ptr(ptr)
    }

    pub fn from_ptr(ptr: *mut u8) -> Self {
        return Self(
            NonNull::new(ptr).expect("expected a valid pointer,got a null pointer instead"),
        );
    }

    pub unsafe fn get(&self, type_info: &TypeInfo, index: usize) -> *mut u8 {
        self.0.as_ptr().add(type_info.layout.size() * index)
    }

    pub unsafe fn set(&mut self, type_info: &TypeInfo, index: usize, data: *mut u8) -> () {
        let dst = self.0.as_ptr().add(type_info.layout.size() * index);

        ptr::copy_nonoverlapping(data, dst, type_info.layout.size());
    }

    pub unsafe fn grow(&mut self, layout: &Layout, old_size: usize, new_size: usize) -> () {
        self.0 = {
            let old_layout = Layout::from_size_align(layout.size() * old_size, layout.align())
                .expect("way to many components");
            let ptr = realloc(self.0.as_ptr(), old_layout, new_size * old_layout.size());
            NonNull::new(ptr).expect("expected a valid pointer,got a null pointer instead")
        };
    }

    pub unsafe fn as_slice(&mut self, size: usize) -> &[*mut u8] {
        let refs = &self.0.as_ptr();
        let ptr = refs as *const *mut u8;
        return std::slice::from_raw_parts(ptr, size);
    }
}

pub struct ArchetypeSet {
    archetypes: FxHashMap<Box<[TypeId]>, Archetype>,
}

impl ArchetypeSet {
    pub fn new() -> Self {
        Self {
            archetypes: FxHashMap::default(),
        }
    }

    pub fn has(&self, types: &[TypeId]) -> bool {
        return self.archetypes.contains_key(types);
    }

    pub fn add(&mut self, types: &[TypeId], type_info: &[TypeInfo]) -> () {
        self.archetypes
            .insert(types.into(), Archetype::new(types, type_info));
    }

    pub fn remove(&mut self, types: &[TypeId]) -> () {
        self.archetypes.remove(types);
    }

    pub fn get(&self, types: &[TypeId]) -> Option<&Archetype> {
        return self.archetypes.get(types);
    }

    pub fn get_mut(&mut self, types: &[TypeId]) -> Option<&mut Archetype> {
        return self.archetypes.get_mut(types);
    }

    pub fn get_by_entity(&self, entity: Entity) -> Option<&Archetype> {
        return self.archetypes.values().filter(|x| x.has(entity)).nth(0);
    }

    pub fn get_by_entity_mut(&mut self, entity: Entity) -> Option<&mut Archetype> {
        return self
            .archetypes
            .values_mut()
            .filter(|x| x.has(entity))
            .nth(0);
    }

    pub fn iter(&mut self) -> impl Iterator<Item = &Archetype> {
        return self.archetypes.values();
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Archetype> {
        return self.archetypes.values_mut();
    }
}

#[cfg(test)]
mod tests {
    use super::{Archetype, ComponentData, TypeInfo};
    use crate::{bundle::Bundle, entity::Entity};
    use std::{any::TypeId, assert_eq};

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
        let archetype = Archetype::new(&type_ids, &type_infos);
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
        let mut archetype = Archetype::new(&type_ids, &type_infos);

        let mut test_data = (3u32, 2u64, TestComponent { a: 1, b: 346 });
        let entity = Entity(3);

        //unsafe { archetype.add(entity, &test_data as *const _ as *mut u8) };
        unsafe { archetype.add(entity, &test_data.as_ptrs()) };

        let t_u32 = unsafe { *(archetype.data[0].get(&type_infos[0], 0) as *mut u32) };
        let t_u64 = unsafe { *(archetype.data[1].get(&type_infos[1], 0) as *mut u64) };
        let test_component =
            unsafe { *(archetype.data[2].get(&type_infos[2], 0) as *mut TestComponent) };

        dbg!(t_u32);
        dbg!(t_u64);
        dbg!(test_component);

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
        let mut archetype = Archetype::new(&type_ids, &type_infos);

        let mut test_data = (3u32, 2u64, TestComponent { a: 1, b: 346 });
        let entity = Entity(3);

        //unsafe { archetype.add(entity, &test_data as *const _ as *mut u8) };
        unsafe { archetype.add(entity, &test_data.as_ptrs()) };

        let rest = unsafe { archetype.remove(entity, &[type_ids[0]]).unwrap() };
        assert_eq!(rest.len(), 2);
        let u = rest[0] as *mut u64;
        let t = rest[1] as *mut TestComponent;
        unsafe {
            assert_eq!(test_data.1, *u);
            assert_eq!(test_data.2, *t);
        }
    }

    #[test]
    fn archetype_grow() {
        let type_ids = [];
        let type_infos = [];
        let mut archetype = Archetype::new(&type_ids, &type_infos);
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
        let mut archetype = Archetype::new(&type_ids, &type_infos);

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
        let mut archetype = Archetype::new(&type_ids, &type_infos);
        let test_data = TestComponent { a: 3, b: 8 };

        let entity = Entity(1);
        drop(archetype);
    }

    #[test]
    fn component_data_set() {
        let type_info = TypeInfo::new::<TestComponent>();
        let layout = type_info.layout;
        let mut data = unsafe { ComponentData::new(layout, 1) };
        let new_size = layout.size() * 5;
        unsafe { data.grow(&layout, 1, new_size) };
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

        let test_components = dbg!(test_components);
        let data = dbg!(data);
        let ts = unsafe {
            Vec::<TestComponent>::from_raw_parts(data.0.as_ptr() as *mut TestComponent, 5, 16)
        };
        assert_eq!(test_components, ts);
    }

    #[test]
    fn component_data_get() {
        let type_info = TypeInfo::new::<TestComponent>();
        let layout = type_info.layout;
        let mut data = unsafe { ComponentData::new(layout, 1) };
        let length = 2;
        let new_size = layout.size() * length;
        unsafe { data.grow(&layout, 1, new_size) };
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
    }

    #[test]
    fn component_data_with_small_types() {
        let type_info = TypeInfo::new::<u32>();
        let layout = type_info.layout;
        let mut data = unsafe { ComponentData::new(layout, 1) };
        let length = 2;
        let new_size = layout.size() * length;
        unsafe { data.grow(&layout, 1, new_size) };
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
    }

    #[test]
    fn component_data_grow() {
        let type_info = TypeInfo::new::<u32>();
        let layout = type_info.layout;
        let mut data = unsafe { ComponentData::new(layout, 1) };
        let mut capacity = 1;
        for _ in 0..12 {
            capacity *= 2;
            unsafe { data.grow(&layout, 1, capacity) };
        }
    }
}
