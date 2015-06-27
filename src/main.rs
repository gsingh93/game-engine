#[macro_use]
extern crate glium;
#[macro_use]
extern crate log;

extern crate env_logger;
extern crate nalgebra;
extern crate time;

mod camera;
mod draw;

use camera::Camera;
use draw::{Cube, Grid};

use glium::{glutin, Display, DisplayBuild, Surface};
use glium::glutin::{ElementState, VirtualKeyCode};

use nalgebra::{BaseFloat, Mat4, Vec3};

const RELATIVE_ROTATION: bool = true;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3]
}

impl Vertex {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vertex { position: [x, y, z] }
    }
}

fn get_rotation_mat(t: time::Timespec) -> Mat4<f32> {
    let sec = (t.sec as f64) + ((t.nsec as f64)/1e9);

    Mat4::new(sec.cos() as f32,  -sec.sin() as f32, 0., 0.,
              sec.sin() as f32,  sec.cos() as f32,  0., 0.,
              0.,                0.,                1., 0.,
              0.,                0.,                0., 1.)
}

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

    implement_vertex!(Vertex, position);

    let mut camera_pos = Vec3::z();
    let mut camera = {
        let (w, h) = get_display_dim(&display);
        let (w, h) = (w as f32, h as f32);
        Camera::new(camera_pos.clone(), w / h)
    };

    let grid = Grid::new(20);
    let grid_req = grid.create_draw_request(&display);

    let cube = Cube;
    let cube_req = cube.create_draw_request(&display);

    let mut mouse_pressed = false;
    let mut old_mouse_coords = None;
    loop {
        let proj_mat = camera.get_projection_matrix();
        let rotate_mat = get_rotation_mat(time::get_time());
        let view_mat = camera.get_view_matrix();
        let grid_uniforms = uniform! { proj_mat: proj_mat, view_mat: view_mat };
        let cube_uniforms = uniform! { proj_mat: proj_mat, view_mat: view_mat,
                                       rotate_mat: rotate_mat };

        let mut target = display.draw();
        target.clear_color_and_depth((0., 0., 0., 1.), 1.);
        draw::draw(&mut target, &grid_req, &grid_uniforms).unwrap();
        draw::draw(&mut target, &cube_req, &cube_uniforms).unwrap();
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
