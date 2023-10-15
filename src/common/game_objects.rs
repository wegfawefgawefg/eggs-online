use glam::Vec2;

pub struct Player {
    pub id: u32,
    pub pos: Vec2,
    pub vel: Vec2,
}
impl Player {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
        }
    }

    pub fn step(&mut self) {
        self.pos += self.vel;
    }
}
