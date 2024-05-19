use perplecs::prelude::*;
use perplecs_macros::Bundle;

#[derive(Bundle)]
struct B {
    a: u32,
}

fn main() {
    let mut w = World::new();
    let e = w.spawn();
    let b = B { a: 1 };
    w.add(e, b);
}
