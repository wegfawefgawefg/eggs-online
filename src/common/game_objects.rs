use glam::Vec2;

pub struct Player {
    pub owner_client_id: u32,
    pub id: u32,
    pub pos: Vec2,
    pub vel: Vec2,
}
impl Player {
    pub fn new(owner_client_id: u32, id: u32) -> Self {
        Self {
            owner_client_id,
            id,
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
        }
    }

    pub fn step(&mut self) {
        self.pos += self.vel;
    }
}
