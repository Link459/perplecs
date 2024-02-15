use std::any::TypeId;

use crate::archetype::TypeInfo;

fn into_ptr<T>(data: &mut T) -> *mut u8 {
    return data as *mut _ as *mut u8;
}

pub trait Bundle: 'static {
    fn type_info() -> Box<[TypeInfo]>;
    fn type_ids() -> Box<[TypeId]>;
    unsafe fn as_ptrs(&mut self) -> Box<[*mut u8]>;
}

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

    unsafe fn as_ptrs(&mut self) -> Box<[*mut u8]> {
        return Box::new([into_ptr(&mut self.0)]);
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

    unsafe fn as_ptrs(&mut self) -> Box<[*mut u8]> {
        return Box::new([into_ptr(&mut self.0), into_ptr(&mut self.1)]);
    }
}

impl<T1, T2, T3> Bundle for (T1, T2, T3)
where
    T1: 'static,
    T2: 'static,
    T3: 'static,
{
    fn type_info() -> Box<[TypeInfo]> {
        Box::new([
            TypeInfo::new::<T1>(),
            TypeInfo::new::<T2>(),
            TypeInfo::new::<T3>(),
        ])
    }

    fn type_ids() -> Box<[TypeId]> {
        return Box::new([TypeId::of::<T1>(), TypeId::of::<T2>(), TypeId::of::<T3>()]);
    }

    unsafe fn as_ptrs(&mut self) -> Box<[*mut u8]> {
        return Box::new([
            into_ptr(&mut self.0),
            into_ptr(&mut self.1),
            into_ptr(&mut self.2),
        ]);
    }
}

impl<T1, T2, T3, T4> Bundle for (T1, T2, T3, T4)
where
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
{
    fn type_info() -> Box<[TypeInfo]> {
        Box::new([
            TypeInfo::new::<T1>(),
            TypeInfo::new::<T2>(),
            TypeInfo::new::<T3>(),
            TypeInfo::new::<T4>(),
        ])
    }

    fn type_ids() -> Box<[TypeId]> {
        return Box::new([
            TypeId::of::<T1>(),
            TypeId::of::<T2>(),
            TypeId::of::<T3>(),
            TypeId::of::<T4>(),
        ]);
    }

    unsafe fn as_ptrs(&mut self) -> Box<[*mut u8]> {
        return Box::new([
            into_ptr(&mut self.0),
            into_ptr(&mut self.1),
            into_ptr(&mut self.2),
            into_ptr(&mut self.3),
        ]);
    }
}

impl<T1, T2, T3, T4, T5> Bundle for (T1, T2, T3, T4, T5)
where
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
    T5: 'static,
{
    fn type_info() -> Box<[TypeInfo]> {
        Box::new([
            TypeInfo::new::<T1>(),
            TypeInfo::new::<T2>(),
            TypeInfo::new::<T3>(),
            TypeInfo::new::<T4>(),
            TypeInfo::new::<T5>(),
        ])
    }

    fn type_ids() -> Box<[TypeId]> {
        return Box::new([
            TypeId::of::<T1>(),
            TypeId::of::<T2>(),
            TypeId::of::<T3>(),
            TypeId::of::<T4>(),
            TypeId::of::<T5>(),
        ]);
    }

    unsafe fn as_ptrs(&mut self) -> Box<[*mut u8]> {
        return Box::new([
            into_ptr(&mut self.0),
            into_ptr(&mut self.1),
            into_ptr(&mut self.2),
            into_ptr(&mut self.3),
            into_ptr(&mut self.4),
        ]);
    }
}

impl<T1, T2, T3, T4, T5, T6> Bundle for (T1, T2, T3, T4, T5, T6)
where
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
    T5: 'static,
    T6: 'static,
{
    fn type_info() -> Box<[TypeInfo]> {
        Box::new([
            TypeInfo::new::<T1>(),
            TypeInfo::new::<T2>(),
            TypeInfo::new::<T3>(),
            TypeInfo::new::<T4>(),
            TypeInfo::new::<T5>(),
            TypeInfo::new::<T6>(),
        ])
    }

    fn type_ids() -> Box<[TypeId]> {
        return Box::new([
            TypeId::of::<T1>(),
            TypeId::of::<T2>(),
            TypeId::of::<T3>(),
            TypeId::of::<T4>(),
            TypeId::of::<T5>(),
            TypeId::of::<T6>(),
        ]);
    }

    unsafe fn as_ptrs(&mut self) -> Box<[*mut u8]> {
        return Box::new([
            into_ptr(&mut self.0),
            into_ptr(&mut self.1),
            into_ptr(&mut self.2),
            into_ptr(&mut self.3),
            into_ptr(&mut self.4),
            into_ptr(&mut self.5),
        ]);
    }
}

impl<T1, T2, T3, T4, T5, T6, T7> Bundle for (T1, T2, T3, T4, T5, T6, T7)
where
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
    T5: 'static,
    T6: 'static,
    T7: 'static,
{
    fn type_info() -> Box<[TypeInfo]> {
        Box::new([
            TypeInfo::new::<T1>(),
            TypeInfo::new::<T2>(),
            TypeInfo::new::<T3>(),
            TypeInfo::new::<T4>(),
            TypeInfo::new::<T5>(),
            TypeInfo::new::<T6>(),
            TypeInfo::new::<T7>(),
        ])
    }

    fn type_ids() -> Box<[TypeId]> {
        return Box::new([
            TypeId::of::<T1>(),
            TypeId::of::<T2>(),
            TypeId::of::<T3>(),
            TypeId::of::<T4>(),
            TypeId::of::<T5>(),
            TypeId::of::<T6>(),
            TypeId::of::<T7>(),
        ]);
    }

    unsafe fn as_ptrs(&mut self) -> Box<[*mut u8]> {
        return Box::new([
            into_ptr(&mut self.0),
            into_ptr(&mut self.1),
            into_ptr(&mut self.2),
            into_ptr(&mut self.3),
            into_ptr(&mut self.4),
            into_ptr(&mut self.5),
            into_ptr(&mut self.6),
        ]);
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8> Bundle for (T1, T2, T3, T4, T5, T6, T7, T8)
where
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
    T5: 'static,
    T6: 'static,
    T7: 'static,
    T8: 'static,
{
    fn type_info() -> Box<[TypeInfo]> {
        Box::new([
            TypeInfo::new::<T1>(),
            TypeInfo::new::<T2>(),
            TypeInfo::new::<T3>(),
            TypeInfo::new::<T4>(),
            TypeInfo::new::<T5>(),
            TypeInfo::new::<T6>(),
            TypeInfo::new::<T7>(),
            TypeInfo::new::<T8>(),
        ])
    }

    fn type_ids() -> Box<[TypeId]> {
        return Box::new([
            TypeId::of::<T1>(),
            TypeId::of::<T2>(),
            TypeId::of::<T3>(),
            TypeId::of::<T4>(),
            TypeId::of::<T5>(),
            TypeId::of::<T6>(),
            TypeId::of::<T7>(),
            TypeId::of::<T8>(),
        ]);
    }

    unsafe fn as_ptrs(&mut self) -> Box<[*mut u8]> {
        return Box::new([
            into_ptr(&mut self.0),
            into_ptr(&mut self.1),
            into_ptr(&mut self.2),
            into_ptr(&mut self.3),
            into_ptr(&mut self.4),
            into_ptr(&mut self.5),
            into_ptr(&mut self.6),
            into_ptr(&mut self.7),
        ]);
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9> Bundle for (T1, T2, T3, T4, T5, T6, T7, T8, T9)
where
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
    T5: 'static,
    T6: 'static,
    T7: 'static,
    T8: 'static,
    T9: 'static,
{
    fn type_info() -> Box<[TypeInfo]> {
        Box::new([
            TypeInfo::new::<T1>(),
            TypeInfo::new::<T2>(),
            TypeInfo::new::<T3>(),
            TypeInfo::new::<T4>(),
            TypeInfo::new::<T5>(),
            TypeInfo::new::<T6>(),
            TypeInfo::new::<T7>(),
            TypeInfo::new::<T8>(),
            TypeInfo::new::<T9>(),
        ])
    }

    fn type_ids() -> Box<[TypeId]> {
        return Box::new([
            TypeId::of::<T1>(),
            TypeId::of::<T2>(),
            TypeId::of::<T3>(),
            TypeId::of::<T4>(),
            TypeId::of::<T5>(),
            TypeId::of::<T6>(),
            TypeId::of::<T7>(),
            TypeId::of::<T8>(),
            TypeId::of::<T9>(),
        ]);
    }

    unsafe fn as_ptrs(&mut self) -> Box<[*mut u8]> {
        return Box::new([
            into_ptr(&mut self.0),
            into_ptr(&mut self.1),
            into_ptr(&mut self.2),
            into_ptr(&mut self.3),
            into_ptr(&mut self.4),
            into_ptr(&mut self.5),
            into_ptr(&mut self.6),
            into_ptr(&mut self.7),
            into_ptr(&mut self.8),
        ]);
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> Bundle for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)
where
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
    T5: 'static,
    T6: 'static,
    T7: 'static,
    T8: 'static,
    T9: 'static,
    T10: 'static,
{
    fn type_info() -> Box<[TypeInfo]> {
        Box::new([
            TypeInfo::new::<T1>(),
            TypeInfo::new::<T2>(),
            TypeInfo::new::<T3>(),
            TypeInfo::new::<T4>(),
            TypeInfo::new::<T5>(),
            TypeInfo::new::<T6>(),
            TypeInfo::new::<T7>(),
            TypeInfo::new::<T8>(),
            TypeInfo::new::<T9>(),
            TypeInfo::new::<T10>(),
        ])
    }

    fn type_ids() -> Box<[TypeId]> {
        return Box::new([
            TypeId::of::<T1>(),
            TypeId::of::<T2>(),
            TypeId::of::<T3>(),
            TypeId::of::<T4>(),
            TypeId::of::<T5>(),
            TypeId::of::<T6>(),
            TypeId::of::<T7>(),
            TypeId::of::<T8>(),
            TypeId::of::<T9>(),
            TypeId::of::<T10>(),
        ]);
    }

    unsafe fn as_ptrs(&mut self) -> Box<[*mut u8]> {
        return Box::new([
            into_ptr(&mut self.0),
            into_ptr(&mut self.1),
            into_ptr(&mut self.2),
            into_ptr(&mut self.3),
            into_ptr(&mut self.4),
            into_ptr(&mut self.5),
            into_ptr(&mut self.6),
            into_ptr(&mut self.7),
            into_ptr(&mut self.8),
            into_ptr(&mut self.9),
        ]);
    }
}
