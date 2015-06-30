#[macro_use]
extern crate glium;
#[macro_use]
extern crate log;

extern crate env_logger;
extern crate image;
extern crate nalgebra;
extern crate obj;
extern crate time;

extern crate genmesh;

mod camera;
mod draw;
mod shader;

use camera::Camera;
use draw::{Cube, Grid};

use glium::{glutin, Display, DisplayBuild, Surface};
use glium::glutin::{ElementState, VirtualKeyCode};

use nalgebra::{zero, BaseFloat, Vec3};

const RELATIVE_ROTATION: bool = true;

fn get_display_dim(display: &Display) -> (u32, u32) {
    match display.get_window().unwrap().get_inner_size() {
        Some(dim) => dim,
        None => panic!("Couldn't get window dimensions")
    }
}

fn main() {
    env_logger::init().unwrap();

    let display = glutin::WindowBuilder::new()
        .with_dimensions(800, 600)
        .with_title(format!("3D Cube"))
        .build_glium()
        .unwrap();

    let mut camera_pos = Vec3::new(0., 0., 1.);
    let mut camera = {
        let (w, h) = get_display_dim(&display);
        let (w, h) = (w as f32, h as f32);
        Camera::new(camera_pos.clone(), w / h)
    };

    let grid = Grid::new(&display, 20);
    let cube = Cube::new(&display, 0.25, zero());

    let mut mouse_pressed = false;
    let mut old_mouse_coords = None;

    let mut ctxt = draw::EngineContext::new();
    loop {
        let mut target = display.draw();
        target.clear_color_and_depth((0., 0., 0., 1.), 1.);
        ctxt.draw(&mut target, &display, &camera, &grid.parent).unwrap();
        ctxt.draw(&mut target, &display, &camera, &cube.parent).unwrap();
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glutin::Event::KeyboardInput(ElementState::Pressed, _, Some(key)) => {
                    match key {
                        VirtualKeyCode::Up => camera_pos.y += 0.05,
                        VirtualKeyCode::Down => camera_pos.y -= 0.05,
                        VirtualKeyCode::Left => camera_pos.x -= 0.05,
                        VirtualKeyCode::Right => camera_pos.x += 0.05,
                        VirtualKeyCode::R => {
                            camera_pos = Vec3::new(0., 0., 1.);
                            camera.set_abs_rotation(0., 0.);
                        }
                        _ => ()
                    }
                    camera.set_pos(&camera_pos);
                },
                glutin::Event::MouseWheel(glutin::MouseScrollDelta::LineDelta(_, v)) => {
                    camera_pos.z += v * 0.05;
                    camera.set_pos(&camera_pos);
                },
                glutin::Event::MouseMoved((x, y)) => {
                    if mouse_pressed {
                        let (x, y) = (x as f32, y as f32);
                        let (w, h) = get_display_dim(&display);
                        let (w, h) = (w as f32, h as f32);
                        if !RELATIVE_ROTATION {
                            let pitch = (y / h) * f32::two_pi();
                            let yaw = (x / w) * f32::two_pi();
                            camera.set_abs_rotation(pitch, -yaw);
                        } else {
                            if let Some((x_old, y_old)) = old_mouse_coords {
                                let delta_x = x - x_old;
                                let delta_y = y - y_old;

                                let pitch = (delta_y * 0.5 / h) * f32::two_pi();
                                let yaw = (delta_x * 0.5 / w) * f32::two_pi();
                                camera.rotate(pitch, yaw);
                            }
                            old_mouse_coords = Some((x, y));
                        }
                    }
                },
                glutin::Event::MouseInput(state, glutin::MouseButton::Left) => {
                    mouse_pressed = if state == ElementState::Pressed {
                        true
                    } else {
                        old_mouse_coords = None;
                        false
                    };
                },
                glutin::Event::Resized(x, y) => {
                    camera.set_aspect_ratio(x as f32 / y as f32);
                },
                glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}
