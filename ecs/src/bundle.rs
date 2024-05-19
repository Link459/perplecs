use std::any::TypeId;

use crate::archetype::TypeInfo;

pub fn into_ptr<T>(data: &mut T) -> *mut u8 {
    return data as *const _ as *mut u8;
}

pub fn from_ptr<'a, T>(data: *mut u8) -> &'a T {
    unsafe { &*(data as *const T) }
}

pub fn from_ptr_mut<'a, T>(data: *mut u8) -> &'a mut T {
    unsafe { &mut *(data as *const T as *mut T) }
}

pub trait Bundle<'a> {
    type Target;
    type TargetMut;
    fn type_info() -> Box<[TypeInfo]>;
    fn type_ids() -> Box<[TypeId]>;
    unsafe fn as_ptrs(&mut self) -> Box<[*mut u8]>;
    unsafe fn from_ptr(data: &[*mut u8]) -> Self::Target;
    unsafe fn from_ptr_mut(data: &[*mut u8]) -> Self::TargetMut;
}

macro_rules! impl_bundle {
    ($($T:ident $I:tt),*)//; $($I:ident),*)
    => {
        impl<'a,$($T),*> Bundle<'a> for ($($T,)*)
            where $($T: 'static ),*
        {

			#[allow(unused_parens)]
            type Target = ($(&'a $T),*);

			#[allow(unused_parens)]
            type TargetMut = ($(&'a mut $T),*);

            fn type_info() -> Box<[TypeInfo]> {
                Box::new([
                         $(TypeInfo::new::<$T>()),*
                ])
            }

			fn type_ids() -> Box<[TypeId]> {
                Box::new([
                         $(TypeId::of::<$T>()),*
                ])
            }

			unsafe fn as_ptrs(&mut self) -> Box<[*mut u8]> {
                Box::new( [
                    //$(into_ptr(&mut self.$I)),*
                    $(into_ptr(&mut self.$I)),*
                ])
            }

			unsafe fn from_ptr(data: &[*mut u8]) -> Self::Target {
               ( $(from_ptr(data[$I])),*  )
           }

			unsafe fn from_ptr_mut(data: &[*mut u8]) -> Self::TargetMut {
               ( $(from_ptr_mut(data[$I])),*  )
           }

        }
    };
}

impl_bundle!(T1 0);
impl_bundle!(T1 0 ,T2 1);
impl_bundle!(T1 0 ,T2 1,T3 2);
impl_bundle!(T1 0 ,T2 1,T3 2,T4 3);
impl_bundle!(T1 0 ,T2 1,T3 2,T4 3,T5 4);
impl_bundle!(T1 0 ,T2 1,T3 2,T4 3,T5 4,T6 5);
impl_bundle!(T1 0 ,T2 1,T3 2,T4 3,T5 4,T6 5,T7 6);
impl_bundle!(T1 0 ,T2 1,T3 2,T4 3,T5 4,T6 5,T7 6,T8 7);
impl_bundle!(T1 0 ,T2 1,T3 2,T4 3,T5 4,T6 5,T7 6,T8 7,T9 8);
impl_bundle!(T1 0 ,T2 1,T3 2,T4 3,T5 4,T6 5,T7 6,T8 7,T9 8,T10 9);
impl_bundle!(T1 0 ,T2 1,T3 2,T4 3,T5 4,T6 5,T7 6,T8 7,T9 8,T10 9,T11 10);
impl_bundle!(T1 0 ,T2 1,T3 2,T4 3,T5 4,T6 5,T7 6,T8 7,T9 8,T10 9,T11 10,T12 11);
impl_bundle!(T1 0 ,T2 1,T3 2,T4 3,T5 4,T6 5,T7 6,T8 7,T9 8,T10 9,T11 10,T12 11,T13 12);
impl_bundle!(T1 0 ,T2 1,T3 2,T4 3,T5 4,T6 5,T7 6,T8 7,T9 8,T10 9,T11 10,T12 11,T13 12,T14 13);
impl_bundle!(T1 0 ,T2 1,T3 2,T4 3,T5 4,T6 5,T7 6,T8 7,T9 8,T10 9,T11 10,T12 11,T13 12,T14 13,T15 14);
