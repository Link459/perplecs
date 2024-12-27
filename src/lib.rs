#![feature(allocator_api)]

pub use perplecs_ecs::*;
pub use perplecs_macros::*;

pub mod prelude {

    pub use perplecs_ecs::{
        bundle::Bundle,
        entity::Entity,
        query::{Query, QueryMut},
        world::World,
    };
    pub use perplecs_macros::Bundle;
    #[cfg(feature = "std")]
    pub type World = perplecs_ecs::world::World<std::alloc::Global>;
}
