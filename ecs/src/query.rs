use crate::{archetype::Archetype, bundle::Bundle};
use std::marker::PhantomData;

pub struct Query<'a, T>
where
    T: Bundle<'a>,
{
    main_archetype: &'a Archetype,
    current_index: usize,
    _phantom_data: PhantomData<T>,
}

impl<'a, T> Query<'a, T>
where
    T: Bundle<'a>,
{
    pub fn new(archetype: &'a Archetype) -> Self {
        Self {
            main_archetype: archetype,
            current_index: 0,
            _phantom_data: PhantomData::default(),
        }
    }
}

impl<'a, T> Iterator for Query<'a, T>
where
    T: Bundle<'a> + 'a,
{
    type Item = T::Target;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.main_archetype.len() {
            return None;
        }

        //let data = unsafe { self.(entity, &T::type_ids())? };
        let data = unsafe {
            self.main_archetype
                .get_by_index(self.current_index, &T::type_ids())
                .unwrap()
        };

        self.current_index += 1;
        return Some(unsafe { T::from_ptr(&data) });
    }
}

pub struct QueryMut<'a, T>
where
    T: Bundle<'a>,
{
    main_archetype: &'a Archetype,
    current_index: usize,
    _phantom_data: PhantomData<T>,
}

impl<'a, T> QueryMut<'a, T>
where
    T: Bundle<'a>,
{
    pub fn new(archetype: &'a Archetype) -> Self {
        Self {
            main_archetype: archetype,
            current_index: 0,
            _phantom_data: PhantomData::default(),
        }
    }
}

impl<'a, T> Iterator for QueryMut<'a, T>
where
    T: Bundle<'a> + 'a,
{
    type Item = &'a mut T::Target;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.main_archetype.len() {
            return None;
        }

        //let data = unsafe { self.(entity, &T::type_ids())? };
        let data = unsafe {
            self.main_archetype
                .get_by_index(self.current_index, &T::type_ids())
                .unwrap()
        };

        self.current_index += 1;
        //return Some(unsafe { &mut T::from_ptr(&data) });
        return None;
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn query_simple() {
    }

    #[test]
    fn query_complex() {}

    fn query_mut() {}
}
