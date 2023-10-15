use client_netcode::spawn_networking_task;
use glam::Vec2;
use glam::{IVec2, UVec2};
use protocol::ClientMessage;
use raylib::prelude::*;
use raylib::{ffi::SetTraceLogLevel, prelude::TraceLogLevel};
use serde::{Deserialize, Serialize};
use settings::SERVER_ADDR;
use sketch::ClientState;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use window::{center_window, scale_and_blit_render_texture_to_window};

mod client_netcode;
mod protocol;
mod settings;
mod sketch;
mod window;

const TIMESTEP: f32 = 1.0 / sketch::FRAMES_PER_SECOND as f32;

#[derive(PartialEq, Eq)]
enum Bool {
    True,
    False,
}

fn main() {
    let state = Arc::new(Mutex::new(ClientState::new()));

    spawn_networking_task(state.clone());

    let (mut rl, mut rlt) = raylib::init().title("raylib-rs-lowres-template").build();
    unsafe {
        SetTraceLogLevel(TraceLogLevel::LOG_WARNING as i32);
    }

    let window_dims = UVec2::new(1280, 720);
    let dims = UVec2::new(240, 160);
    let fullscreen = false;
    rl.set_window_size(window_dims.x as i32, window_dims.y as i32);
    if fullscreen {
        rl.toggle_fullscreen();
        rl.set_window_size(rl.get_screen_width(), rl.get_screen_height());
    }

    center_window(&mut rl, window_dims);
    let mouse_scale = dims.as_vec2() / window_dims.as_vec2();
    rl.set_mouse_scale(mouse_scale.x, mouse_scale.y);

    let mut render_texture = rl
        .load_render_texture(&rlt, dims.x, dims.y)
        .unwrap_or_else(|e| {
            println!("Error creating render texture: {}", e);
            std::process::exit(1);
        });

    while !rl.window_should_close() {
        let should_continue;

        // Process user input and events within its own scope
        {
            let mut locked_state = match state.lock() {
                Ok(locked) => locked,
                Err(_) => {
                    println!("Client step loop: Failed to acquire lock on state.");
                    continue; // Skip the current iteration
                }
            };
            sketch::process_events_and_input(&mut rl, &mut locked_state);
            should_continue = if locked_state.running {
                Bool::True
            } else {
                Bool::False
            };
        }

        // Process any received network data, applying it to the state
        // process_network_data(&mut state);

        // Update and render within its own scope
        {
            let mut locked_state = state.lock().unwrap();

            // Use predictive rendering based on local inputs and last known state
            let dt = rl.get_frame_time();
            locked_state.time_since_last_update += dt;
            while locked_state.time_since_last_update > TIMESTEP {
                locked_state.time_since_last_update -= TIMESTEP;

                sketch::step(&mut rl, &mut rlt, &mut locked_state);
            }

            // Render
            let mut draw_handle = rl.begin_drawing(&rlt);
            {
                let low_res_draw_handle =
                    &mut draw_handle.begin_texture_mode(&rlt, &mut render_texture);
                low_res_draw_handle.clear_background(Color::BLACK);
                sketch::draw(&locked_state, low_res_draw_handle);
            }
            scale_and_blit_render_texture_to_window(
                &mut draw_handle,
                &mut render_texture,
                fullscreen,
                window_dims,
            );
        }

        // If we should not continue, break out of the loop
        if should_continue == Bool::False {
            break;
        }
    }
}
