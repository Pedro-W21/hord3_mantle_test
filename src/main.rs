#![feature(portable_simd)]
#![feature(int_roundings)]
#![feature(mpmc_channel)]
#[allow(warnings)]

use std::{f32::consts::PI, sync::mpmc::channel, thread, time::Duration};

use hord3::{defaults::default_rendering::mantle::{api::{ApiLod, CPUInstanceData, CPUVertexData, MantleEvent, MantleRequest}, meshes::{IndexData, MeshID}, textures::{buffer_to_request, load_single_texture}, winit_app::App}, horde::{geometry::{mat4::Mat4, rotation::{Orientation, Rotation}, vec3d::{Vec3D, Vec3Df}}, rendering::camera::Camera}};
use winit::event_loop::EventLoop;

use crate::{main_copy::run_game, main_turbo::turbo_run_game};
pub mod entity;
pub mod engine;
pub mod tasks;
pub mod main_copy;


pub mod game_map;
pub mod game_entity;
pub mod game_engine;
pub mod game_tasks;
pub mod game_input_handler;
pub mod tile_editor;
pub mod gui_elements;
pub mod game_3d_models;
pub mod shared;
pub mod entities;
pub mod turbo_game_engine;
pub mod main_turbo;
pub mod turbo_game_tasks;

pub mod turbo_tests;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let (mut app,handler) = App::new(&event_loop, 100_000_000, 100_000_000, 10_000_000);
    thread::spawn(move || {
        turbo_run_game(handler);
    });
    event_loop.run_app(&mut app);
}
