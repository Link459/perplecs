use std::{
    ops::{Add, AddAssign, Mul},
    time::Instant,
};

use perplecs::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Add<Vec2> for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) -> () {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
struct Rigidbody {
    position: Vec2,
    velocity: Vec2,
    angle: f32,
    angular_velocity: f32,
    torque: f32,
    inertia: f32,
    mass: f32,
}

fn apply_dynamics(world: &mut World, dt: f32) {
    for rb in world.query_mut::<(Rigidbody,)>() {
        let force = Vec2::new(0.0, rb.mass * -9.81);
        let acceleration = Vec2::new(force.x / rb.mass, force.y / rb.mass);
        rb.velocity += acceleration * dt;
        rb.position += rb.velocity * dt;
        let angular_acceleration = rb.torque / rb.inertia;
        rb.angular_velocity += angular_acceleration * dt;
        rb.angle += rb.angular_velocity * dt;
    }
}

fn main() -> () {
    let mut world = World::new();
    let particle = world.spawn();
    world.add(particle, (Rigidbody::default(),));
    let rigidbody = world.get_mut::<(Rigidbody,)>(particle).unwrap();
    rigidbody.mass = 0.1;
    let mut last_time = Instant::now();
    for _ in 0..10 {
        let current_time = Instant::now();
        let dt = current_time.duration_since(last_time).as_secs_f32();
        apply_dynamics(&mut world, dt);
        last_time = current_time;
    }
}
