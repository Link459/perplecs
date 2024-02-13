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
        for i in 0..type_ids.len() {
            data.push(unsafe { Self::alloc(&archetype, type_ids[i]) });
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

    pub fn add(&mut self, entity: Entity, data: *mut u8) -> () {
        if self.entities.len() == self.capacity {
            unsafe { self.grow(self.capacity * 2) };
        }

        self.entities.push(entity);
        //TODO: add the components to the data storage
    }

    pub fn remove(&mut self, entity: Entity) -> () {
        //TODO: handle case where there isn't a entity in it
        let index = self.entities.binary_search(&entity).unwrap();
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

    pub fn get<'a>(&self, entity: Entity, type_id: &TypeId) -> Option<NonNull<u8>> {
        let index = self.entities.binary_search(&entity).ok()?;
        let ty_index = self.type_ids.binary_search(type_id).ok()?;
        let ty = &self.types[type_id];
        return unsafe {
            Some(NonNull::new(self.data[ty_index].get(ty, index)).expect("ptr is null"))
        };
    }

    unsafe fn alloc(&self, id: TypeId) -> ComponentData {
        let info = self.types.get(&id).expect("invalid type");
        return ComponentData::new(info.layout);
    }

    unsafe fn grow(&mut self, new_size: usize) {
        for (data, ty) in self.data.iter_mut().zip(self.type_ids.iter()) {
            let ty = self.types[ty];
            data.grow(&ty.layout, new_size);
        }
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
    pub unsafe fn new(layout: Layout) -> Self {
        let ptr = alloc(layout);
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
        let dest = self.0.as_ptr().add(type_info.layout.size() * index);

        ptr::copy(data, dest, type_info.layout.size());
    }

    pub unsafe fn grow(&mut self, layout: &Layout, new_size: usize) -> () {
        self.0 = {
            let ptr = realloc(self.0.as_ptr(), *layout, new_size);
            NonNull::new(ptr).expect("expected a valid pointer,got a null pointer instead")
        };
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
    use crate::entity::Entity;
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
            TypeId::of::<f64>(),
            TypeId::of::<TestComponent>(),
        ];
        let type_infos = [
            TypeInfo::new::<u32>(),
            TypeInfo::new::<f64>(),
            TypeInfo::new::<TestComponent>(),
        ];
        let archetype = Archetype::new(&type_ids, &type_infos);

        //TODO: finish the test
    }

    #[test]
    fn archetype_remove() {}

    #[test]
    fn archetype_get() {}

    #[test]
    fn archetype_drop() {}

    #[test]
    fn component_data_set() {
        let type_info = TypeInfo::new::<TestComponent>();
        let layout = type_info.layout;
        let mut data = unsafe { ComponentData::new(layout) };
        let new_size = layout.size() * 5;
        unsafe { data.grow(&layout, new_size) };
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
        let mut data = unsafe { ComponentData::new(layout) };
        let length = 2;
        let new_size = layout.size() * length;
        unsafe { data.grow(&layout, new_size) };
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
}
