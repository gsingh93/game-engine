#[macro_use]
extern crate glium;
#[macro_use]
extern crate log;

extern crate env_logger;
extern crate freetype;
extern crate genmesh;
extern crate image;
extern crate nalgebra;
extern crate obj;
extern crate time;

mod camera;
mod draw;
mod shader;

use std::{mem, thread};
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Read;
use std::fs::File;
use std::rc::Rc;

use camera::Camera;
use draw::{Cube, Grid, GameObject, Text};
use shader::{ShaderType, FragmentShaderType, VertexShaderType};

use freetype as ft;

use glium::{glutin, Display, DisplayBuild, DrawError, Program, Surface};
use glium::glutin::{ElementState, VirtualKeyCode};
use glium::texture::{ClientFormat, RawImage2d, Texture2d};

use nalgebra::{zero, BaseFloat, Vec3};

struct Scene<'a> {
    // TODO: Do we want this to be GameObject + 'a?
    named_objects: HashMap<String, Box<GameObject + 'a>>,
    unamed_objects: Vec<Box<GameObject + 'a>>,
    camera: Camera,
}

impl<'a> Scene<'a> {
    fn new(camera: Camera) -> Self {
        Scene { camera: camera, named_objects: HashMap::new(), unamed_objects: Vec::new() }
    }

    fn draw(&self, ctxt: &mut EngineContext) {
        let mut target = ctxt.display.draw();
        target.clear_color_and_depth((0., 0., 0., 1.), 1.);
        for obj in self.named_objects.values().chain(self.unamed_objects.iter()) {
            ctxt.draw(&mut target, &self.camera, obj).unwrap();
        }
        target.finish().unwrap();
    }

    fn add<G: GameObject + 'a>(&mut self, object: G) {
        if object.name().is_empty() {
            self.unamed_objects.push(Box::new(object));
        } else {
            assert!(self.named_objects.insert(object.name().to_owned(),
                                              Box::new(object)).is_none(),
                    "Duplicate object name");
        }
    }

    fn add_text(&mut self, text: Text<'static>) {
        for c in text.into_chars() {
            self.add(c);
        }
    }

    unsafe fn get_object<T: GameObject>(&self, name: &str) -> Option<&T> {
        self.named_objects.get(name).map(|o| mem::transmute(o))
    }
}

const RELATIVE_ROTATION: bool = true;

pub fn get_display_dim(display: &Display) -> (u32, u32) {
    match display.get_window().unwrap().get_inner_size() {
        Some(dim) => dim,
        None => panic!("Couldn't get window dimensions")
    }
}

pub struct EngineContext {
    display: Display,
    vert_shader_map: HashMap<VertexShaderType, String>,
    frag_shader_map: HashMap<FragmentShaderType, String>,
    texture_cache: TextureCache,
}

pub struct TextureCache {
    cache: HashMap<&'static str, Rc<Texture2d>>,
    glyph_cache: HashMap<char, Rc<Character>>,
}

pub struct Character {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    advance_x: f32,
    advance_y: f32,
    texture: Texture2d,
}

impl TextureCache {
    fn new() -> Self {
        TextureCache { cache: HashMap::new(), glyph_cache: HashMap::new() }
    }

    fn get_texture(&mut self, display: &Display, name: &'static str) -> Rc<Texture2d> {
        self.cache.entry(name).or_insert_with(|| {
            let f = File::open(name).unwrap();
            let image = image::load(f, image::PNG).unwrap();
            Rc::new(Texture2d::new(display, image))
        }).clone()
    }

    fn get_glyph(&mut self, display: &Display, face: &ft::Face, c: char) -> Rc<Character> {
        self.glyph_cache.entry(c).or_insert_with(|| {
            face.load_char(c as usize, ft::face::RENDER).unwrap();
            let g = face.glyph();

            let bitmap = g.bitmap();
            Rc::new(Character {
                left: g.bitmap_left() as f32,
                top: g.bitmap_top() as f32,
                width: bitmap.width() as f32,
                height: bitmap.rows() as f32,
                advance_x: (g.advance().x >> 6) as f32,
                advance_y: (g.advance().y >> 6) as f32,
                texture: Texture2d::new(display, RawImage2d {
                    data: Cow::Borrowed(bitmap.buffer()),
                    width: bitmap.width() as u32, height: bitmap.rows() as u32,
                    format: ClientFormat::U8
                })
            })
        }).clone()
    }
}

impl EngineContext {
    pub fn new(display: Display) -> Self {
        EngineContext { display: display, vert_shader_map: HashMap::new(),
                        frag_shader_map: HashMap::new(), texture_cache: TextureCache::new() }
    }

    pub fn draw<S: Surface>(&mut self, surface: &mut S, camera: &Camera,
                            obj: &Box<GameObject>) -> Result<(), DrawError> {
        let parent = obj.parent();

        let &mut EngineContext { ref display, ref mut vert_shader_map,
                                 ref mut frag_shader_map, .. } = self;
        let vertex_shader = Self::get_shader(vert_shader_map, parent.vert_shader_type);
        let fragment_shader = Self::get_shader(frag_shader_map, parent.frag_shader_type);
        let program = Program::from_source(display, vertex_shader, fragment_shader, None).unwrap();

        let uniforms = obj.construct_uniforms(&camera);

        surface.draw(&parent.vertex_buffer, parent.indices.clone(), &program, &uniforms,
                     &parent.draw_params)
    }

    fn get_shader<S: ShaderType>(shader_map: &mut HashMap<S, String>, shader_type: S) -> &str {
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

    let camera = {
        let (w, h) = get_display_dim(&display);
        let (w, h) = (w as f32, h as f32);
        Camera::new(Vec3::new(0., 0., 1.), w / h)
    };

    let mut ctxt = EngineContext::new(display);

    let mut scene = Scene::new(camera);
    let mut g = Grid::new(&ctxt.display, 20);
    g.parent.name = "grid".to_owned();
    scene.add(g);
    scene.add(Cube::new(&mut ctxt, 1., zero()));

    // FIXME: Text needs to go last
    scene.add_text(Text::new(&mut ctxt, -0.9, -0.9, "Frame rate: 60fps"));

    let mut right_mouse_pressed = false;
    let mut left_mouse_pressed = false;
    let mut old_mouse_coords = None;

    let mut accumulator = 0;
    let mut previous_time = time::precise_time_ns();
    loop {
        for ev in ctxt.display.poll_events() {
            match ev {
                glutin::Event::KeyboardInput(ElementState::Pressed, _, Some(key)) => {
                    match key {
                        VirtualKeyCode::R => {
                            scene.camera.set_pos(&Vec3::new(0., 0., 1.));
                            scene.camera.set_abs_rotation(0., 0.);
                            scene.camera.set_fov(BaseFloat::frac_pi_2());
                        }
                        _ => ()
                    }
                },
                glutin::Event::MouseWheel(glutin::MouseScrollDelta::LineDelta(_, v)) => {
                    let fov = scene.camera.fov();
                    let frac: f32 = (f32::pi() - fov) / f32::pi();
                    let new_fov = f32::max(0., fov + 0.05 * frac * v);
                    scene.camera.set_fov(new_fov);
                },
                glutin::Event::MouseMoved((x, y)) => {
                    if right_mouse_pressed {
                        // Rotation
                        let (x, y) = (x as f32, y as f32);
                        let (w, h) = get_display_dim(&ctxt.display);
                        let (w, h) = (w as f32, h as f32);
                        if !RELATIVE_ROTATION {
                            let pitch = (y / h) * f32::two_pi();
                            let yaw = (x / w) * f32::two_pi();
                            scene.camera.set_abs_rotation(pitch, -yaw);
                        } else {
                            if let Some((x_old, y_old)) = old_mouse_coords {
                                let delta_x = x - x_old;
                                let delta_y = y - y_old;

                                let pitch = (delta_y * 0.5 / h) * f32::two_pi();
                                let yaw = (delta_x * 0.5 / w) * f32::two_pi();
                                scene.camera.rotate(pitch, yaw);
                            }
                            old_mouse_coords = Some((x, y));
                        }
                    } else if left_mouse_pressed {
                        // Translation
                        let (x, y) = (x as f32, y as f32);
                        if let Some((x_old, y_old)) = old_mouse_coords {
                            let diff = Vec3::new(x_old - x, y - y_old, 0.) * 0.003 as f32;
                            scene.camera.translate(&diff);
                        }
                        old_mouse_coords = Some((x, y));
                    }
                },
                glutin::Event::MouseInput(state, button) => {
                    if state == ElementState::Released {
                        old_mouse_coords = None;
                    };

                    match button {
                        glutin::MouseButton::Left =>
                            left_mouse_pressed = state == ElementState::Pressed,
                        glutin::MouseButton::Right =>
                            right_mouse_pressed = state == ElementState::Pressed,
                        _ => ()
                    }
                }
                glutin::Event::Resized(x, y) => {
                    scene.camera.set_aspect_ratio(x as f32 / y as f32);
                },
                glutin::Event::Closed => return,
                _ => ()
            }
        }

        let now = time::precise_time_ns();
        let delta = now - previous_time;
        accumulator += delta;
        previous_time = now;

        const FPS: u64 = 60;
        const FIXED_TIME_STAMP: u64 = 10e9 as u64 / FPS;
        if accumulator >= FIXED_TIME_STAMP {
            while accumulator >= FIXED_TIME_STAMP {
                accumulator -= FIXED_TIME_STAMP;
                // TODO: Update
            }
            scene.draw(&mut ctxt);
        }

        // A half-assed attempt to not use up the entire CPU and still keep the frame rate
        let sleep_time = ((FIXED_TIME_STAMP - accumulator) / 1000000) as i32;
        if sleep_time > 30 {
            let sleep_time = sleep_time * 3 / 5;
            thread::sleep_ms(sleep_time as u32);
        }
    }
}
