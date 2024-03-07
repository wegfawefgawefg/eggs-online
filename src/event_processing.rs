use raylib::prelude::*;

use crate::state::State;

const PLAYER_SPEED: f32 = 1.0;

pub fn process_events_and_input(rl: &mut RaylibHandle, state: &mut State) {
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ESCAPE) {
        state.running = false;
    }

    for (eid, mut player) in &mut state.players {
        if let Some(client_id) = state.client_id {
            if player.owner_client_id != client_id {
                continue;
            }
        }

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
    }
}
