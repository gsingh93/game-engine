use Vertex;

use glium::{Display, DrawError, Surface, Program, VertexBuffer};
use glium::draw_parameters::DrawParameters;
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

pub fn draw_grid(display: &Display) -> DrawRequest {
    let vertex_shader_src = r#"
    #version 140

    in vec3 position;
    uniform mat4 proj_mat;

    void main() {
        gl_Position = proj_mat * vec4(position, 1.);
    }
"#;

    let fragment_shader_src = r#"
    #version 140

    out vec4 color;

    void main() {
        color = vec4(1.);
    }
"#;

    let mut shape = Vec::new();
    for i in (-10..10) {
        let v1 = Vertex::new(i as f32 / 10., -1., 0.);
        let v2 = Vertex::new(i as f32 / 10., 1., -1.);

        let v3 = Vertex::new(-1., i as f32 / 10., -1.);
        let v4 = Vertex::new(1., i as f32 / 10., -1.);

        shape.push(v1);
        shape.push(v2);
        shape.push(v3);
        shape.push(v4);
    }

    let vb = VertexBuffer::new(display, shape);
    let indices = NoIndices(PrimitiveType::LinesList);
    let program = Program::from_source(display, vertex_shader_src,
                                       fragment_shader_src, None).unwrap();

    DrawRequest::new(vb, indices, program, Default::default())
}

pub fn draw<T: Surface, U: Uniforms>(target: &mut T, draw_req: &DrawRequest, uniforms: &U)
                    -> Result<(), DrawError> {
    let &DrawRequest { ref vb, ref indices, ref program, ref params } = draw_req;

    target.draw(vb, indices, program, uniforms, params)
}
