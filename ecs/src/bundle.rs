use std::any::TypeId;

use crate::archetype::TypeInfo;

pub trait Bundle: 'static {
    fn type_info() -> Box<[TypeInfo]>;
    fn type_ids() -> Box<[TypeId]>;
}

//TODO: impl bundle for more tuples

impl<T> Bundle for (T,)
where
    T: 'static,
{
    fn type_info() -> Box<[TypeInfo]> {
        Box::new([TypeInfo::new::<T>()])
    }
    fn type_ids() -> Box<[TypeId]> {
        return Box::new([TypeId::of::<T>()]);
    }
}

impl<T1, T2> Bundle for (T1, T2)
where
    T1: 'static,
    T2: 'static,
{
    fn type_info() -> Box<[TypeInfo]> {
        Box::new([TypeInfo::new::<T1>(), TypeInfo::new::<T2>()])
    }

    fn type_ids() -> Box<[TypeId]> {
        return Box::new([TypeId::of::<T1>(), TypeId::of::<T2>()]);
    }
}
