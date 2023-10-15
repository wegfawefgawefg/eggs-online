use std::collections::HashMap;

use glam::Vec2;
use raylib::prelude::*;

use crate::protocol::Player;

pub const FRAMES_PER_SECOND: u32 = 60;

pub struct ClientState {
    pub running: bool,
    pub time_since_last_update: f32,
    pub player_id: Option<u32>,
    pub players: HashMap<u32, Player>,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            running: true,
            time_since_last_update: 0.0,

            player_id: None,

            players: HashMap::new(),
        }
    }
}

const PLAYER_SPEED: f32 = 1.0;

pub fn process_events_and_input(rl: &mut RaylibHandle, state: &mut ClientState) {
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ESCAPE) {
        state.running = false;
    }

    if let Some(id) = state.player_id {
        if let Some(player) = state.players.get_mut(&id) {
            // Handle player movement with WASD keys
            if rl.is_key_down(raylib::consts::KeyboardKey::KEY_W) {
                player.pos.y -= PLAYER_SPEED;
            }
            if rl.is_key_down(raylib::consts::KeyboardKey::KEY_S) {
                player.pos.y += PLAYER_SPEED;
            }
            if rl.is_key_down(raylib::consts::KeyboardKey::KEY_A) {
                player.pos.x -= PLAYER_SPEED;
            }
            if rl.is_key_down(raylib::consts::KeyboardKey::KEY_D) {
                player.pos.x += PLAYER_SPEED;
            }

            // print the player pos
            println!("Player pos: {}", player.pos);
        }
    }
}

pub fn step(rl: &mut RaylibHandle, rlt: &mut RaylibThread, state: &mut ClientState) {
    // set the mouse
}

pub fn draw(state: &ClientState, d: &mut RaylibTextureMode<RaylibDrawHandle>) {
    d.draw_text("Multiplayer!", 12, 12, 12, Color::WHITE);
    let mouse_pos = d.get_mouse_position();
    d.draw_circle(mouse_pos.x as i32, mouse_pos.y as i32, 6.0, Color::GREEN);

    // render the player
    // d.draw_circle(
    //     state.player_pos.x as i32,
    //     state.player_pos.y as i32,
    //     6.0,
    //     Color::BLUE,
    // );

    // render all players
    for (_, player) in state.players.iter() {
        // color is a hash of the player id
        let color = Color::new(
            (player.id as u8).wrapping_mul(17),
            (player.id as u8).wrapping_mul(23),
            (player.id as u8).wrapping_mul(29),
            255,
        );
        d.draw_circle(player.pos.x as i32, player.pos.y as i32, 6.0, Color::BLUE);
    }
}
