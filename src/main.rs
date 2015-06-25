#[macro_use]
extern crate glium;

extern crate nalgebra;

mod camera;
mod draw;

use camera::Camera;

use glium::{glutin, DisplayBuild, Surface};

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3]
}

impl Vertex {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vertex { position: [x, y, z] }
    }
}

fn main() {
    let display = glutin::WindowBuilder::new()
        .with_dimensions(800, 600)
        .with_title(format!("3D Cube"))
        .build_glium()
        .unwrap();

    implement_vertex!(Vertex, position);

    let camera = Camera::new();
    let proj_mat = camera.get_projection_matrix();
    let view_mat = camera.get_view_matrix();
    let grid_req = draw::draw_grid(&display);

    loop {
        let uniforms = uniform! { proj_mat: proj_mat };

        let mut target = display.draw();
        target.clear_color_and_depth((0., 0., 1., 1.), 1.);
        draw::draw(&mut target, &grid_req, &uniforms).unwrap();
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}
