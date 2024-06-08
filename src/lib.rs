#![feature(allocator_api)]

pub use perplecs_ecs::*;
pub use perplecs_macros::*;

pub mod prelude {
    use std::alloc::Global;

    pub use perplecs_ecs::bundle::Bundle;
    pub use perplecs_macros::Bundle;
    pub type World = perplecs_ecs::world::World<Global>;
}
