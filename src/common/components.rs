use glam::Vec2;
use hecs::Entity;
use raylib::prelude::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CTransform {
    pub pos: Vec2,
    pub rot: Vec2,
}

pub struct Color {
    pub color: Color,
}

#[derive(Clone, Copy)]
pub struct Shape {
    pub dims: Vec2,
}

#[derive(Clone, Copy)]
pub struct Physics {
    pub vel: Vec2,
}

pub struct RegistryRecord {
    pub client_id: u32,
    pub entity: Entity,
}

pub struct Registry {
    pub records: Vec<RegistryRecord>,
}
