use Vertex;

use glium::{DepthTest, Display, DrawError, DrawParameters, Surface, Program, VertexBuffer};
use glium::index::{NoIndices, PrimitiveType};
use glium::uniforms::Uniforms;

pub struct DrawRequest<'a> {
    vb: VertexBuffer<Vertex>,
    indices: NoIndices,
    program: Program,
    params: DrawParameters<'a>,
}

impl<'a> DrawRequest<'a> {
    fn new(vb: VertexBuffer<Vertex>, indices: NoIndices, program: Program,
           params: DrawParameters<'a>) -> Self {
        DrawRequest { vb: vb, indices: indices, program: program, params: params }
    }
}

pub struct Grid;

impl Grid {
    pub fn create_draw_request<'a>(&self, display: &Display) -> DrawRequest<'a> {
        let vertex_shader_src = r#"
#version 140

in vec3 position;
uniform mat4 proj_mat;
uniform mat4 view_mat;

void main() {
    gl_Position = proj_mat * view_mat * vec4(position, 1.);
}"#;

        let fragment_shader_src = r#"
#version 140

out vec4 color;

void main() {
    color = vec4(1.);
}"#;

        let mut shape = Vec::new();
        for i in (-10..10) {
            let v1 = Vertex::new(i as f32 / 10., -1., 0.);
            let v2 = Vertex::new(i as f32 / 10., 1., 0.);

            let v3 = Vertex::new(-1., i as f32 / 10., 0.);
            let v4 = Vertex::new(1., i as f32 / 10., 0.);

            shape.push(v1);
            shape.push(v2);
            shape.push(v3);
            shape.push(v4);
        }

        let params = DrawParameters {
            depth_test: DepthTest::IfLess,
            depth_write: true,
            .. Default::default()
        };

        let vb = VertexBuffer::new(display, shape);
        let indices = NoIndices(PrimitiveType::LinesList);
        let program = Program::from_source(display, vertex_shader_src,
                                           fragment_shader_src, None).unwrap();

        DrawRequest::new(vb, indices, program, params)
    }
}

pub struct Cube;

impl Cube {
    pub fn create_draw_request<'a>(&self, display: &Display) -> DrawRequest<'a> {
        let vertex_shader_src = r#"
#version 140

in vec3 position;
uniform mat4 proj_mat;
uniform mat4 view_mat;
uniform mat4 rotate_mat;

out vec3 v_coord;

void main() {
    v_coord = position;
    gl_Position = proj_mat * view_mat * rotate_mat * vec4(position, 1.);
}"#;

        let fragment_shader_src = r#"
#version 140

in vec3 v_coord;
out vec4 color;

void main() {
    if (abs(v_coord.z) == 0.25) {
        color = vec4(0., 1., 0., 1.);
    } else if (abs(v_coord.x) == 0.25) {
        color = vec4(0., 0., 1., 1.);
    } else if (abs(v_coord.y) == 0.25) {
        color = vec4(1., 0., 0., 1.);
    }
}"#;

        let v1 = Vertex::new(0.25, 0.25, -0.25);
        let v2 = Vertex::new(-0.25, 0.25, -0.25);
        let v3 = Vertex::new(0.25, -0.25, -0.25);
        let v4 = Vertex::new(-0.25, -0.25, -0.25);
        let v5 = Vertex::new(-0.25, 0.25, 0.25);
        let v6 = Vertex::new(-0.25, -0.25, 0.25);
        let v7 = Vertex::new(0.25, -0.25, 0.25);
        let v8 = Vertex::new(0.25, 0.25, 0.25);

        let shape = vec![
            // Back face (z = -.25)
            v1, v2, v3,
            v2, v3, v4,

            // Left face (x = -.25)
            v2, v4, v5,
            v4, v5, v6,

            // Bottom face (y = -.25)
            v4, v6, v7,
            v3, v4, v7,

            // Front face (z = .25)
            v5, v6, v7,
            v5, v7, v8,

            // Right face (x = .25)
            v3, v7, v8,
            v1, v3, v8,

            // Top face (y = .25)
            v1, v2, v5,
            v1, v5, v8,
        ];

        let params = DrawParameters {
            depth_test: DepthTest::IfLess,
            depth_write: true,
            .. Default::default()
        };

        let vb = VertexBuffer::new(display, shape);
        let indices = NoIndices(PrimitiveType::TriangleStrip);
        let program = Program::from_source(display, vertex_shader_src,
                                           fragment_shader_src, None).unwrap();

        DrawRequest::new(vb, indices, program, params)
    }
}

pub fn draw<T: Surface, U: Uniforms>(target: &mut T, draw_req: &DrawRequest, uniforms: &U)
                    -> Result<(), DrawError> {
    let &DrawRequest { ref vb, ref indices, ref program, ref params } = draw_req;

    target.draw(vb, indices, program, uniforms, params)
}
