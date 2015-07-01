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

use std::collections::HashMap;
use std::io::Read;
use std::fs::File;

use camera::Camera;
use draw::{Cube, Grid, GameObject};
use shader::ShaderType;

use glium::{glutin, Display, DisplayBuild, DrawError, Program, Surface};
use glium::backend::Facade;
use glium::glutin::{ElementState, VirtualKeyCode};

use nalgebra::{zero, BaseFloat, Vec3};

type Scene = Vec<Box<GameObject>>;

const RELATIVE_ROTATION: bool = true;

fn get_display_dim(display: &Display) -> (u32, u32) {
    match display.get_window().unwrap().get_inner_size() {
        Some(dim) => dim,
        None => panic!("Couldn't get window dimensions")
    }
}

pub struct EngineContext {
    vertex_shader: String,
    shader_map: HashMap<ShaderType, String>,
}

impl EngineContext {
    pub fn new() -> Self {
        let mut shader = String::new();
        File::open("shaders/vertex.glsl").unwrap().read_to_string(&mut shader).unwrap();
        EngineContext { vertex_shader: shader, shader_map: HashMap::new() }
    }

    pub fn draw<'a, F: Facade, S: Surface>(&mut self,
                                           surface: &mut S,
                                           facade: &F,
                                           camera: &Camera,
                                           obj: &Box<GameObject>)
                                           -> Result<(), DrawError> {
        let parent = obj.parent();
        let uniforms = obj.construct_uniforms(&camera);

        let &mut EngineContext { ref vertex_shader, ref mut shader_map } = self;
        let fragment_shader = Self::get_shader(shader_map, parent.shader_type);
        let program = Program::from_source(facade, vertex_shader,
                                           fragment_shader, None).unwrap();
        surface.draw(&parent.vertex_buffer, parent.indices.clone(), &program, &uniforms,
                     &parent.draw_params)
    }

    fn get_shader(shader_map: &mut HashMap<ShaderType, String>, shader_type: ShaderType) -> &str {
        shader_map.entry(shader_type).or_insert_with(|| {
            let mut shader = String::new();
            File::open(shader_type.to_filename()).unwrap().read_to_string(&mut shader).unwrap();
            shader
        })
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

    let mut scene = Scene::new();
    scene.push(Box::new(Grid::new(&display, 20)));
    scene.push(Box::new(Cube::new(&display, 0.25, zero())));

    let mut mouse_pressed = false;
    let mut old_mouse_coords = None;

    let mut ctxt = EngineContext::new();
    loop {
        let mut target = display.draw();
        target.clear_color_and_depth((0., 0., 0., 1.), 1.);
        for obj in scene.iter() {
            ctxt.draw(&mut target, &display, &camera, obj).unwrap();
        }
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
                            camera.set_fov(BaseFloat::frac_pi_2());
                        }
                        _ => ()
                    }
                    camera.set_pos(&camera_pos);
                },
                glutin::Event::MouseWheel(glutin::MouseScrollDelta::LineDelta(_, v)) => {
                    let fov = camera.fov();
                    let new_fov = nalgebra::clamp(fov + (v * 0.05), 0., BaseFloat::pi());
                    camera.set_fov(new_fov);
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
