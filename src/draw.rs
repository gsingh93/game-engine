use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::fs::File;
use std::path::Path;

use shader::ShaderType;
use camera::Camera;

use genmesh;

use glium::{DepthTest, DrawError, DrawParameters, Surface, Program, VertexBuffer};
use glium::backend::Facade;
use glium::index::{IndicesSource, NoIndices, PrimitiveType};
use glium::vertex::VertexBufferAny;

use nalgebra::{self, Vec3, Mat4};

use obj;

use time;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
}

impl Vertex {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vertex { position: [x, y, z], tex_coord: [0., 0.] }
    }
}

implement_vertex!(Vertex, position, tex_coord);

pub struct ObjectBuilder<'a> {
    vertex_buffer: VertexBufferAny,
    indices: IndicesSource<'a>,
    draw_params: Option<DrawParameters<'a>>,
    transform: Option<Mat4<f32>>,
}

impl<'a> ObjectBuilder<'a> {
    pub fn new<I: Into<IndicesSource<'a>>>(vb: VertexBufferAny, indices: I) -> Self {
        ObjectBuilder {
            vertex_buffer: vb,
            indices: indices.into(),
            draw_params: None,
            transform: None,
        }
    }

    pub fn from_obj<F, I, P>(facade: &F, path: P, indices: I) -> Self
    where F: Facade, I: Into<IndicesSource<'a>>, P: AsRef<Path> {
        let vb = load_obj(facade, &mut BufReader::new(File::open(path).unwrap()));
        Self::new(vb, indices)
    }

    pub fn draw_params(mut self, params: DrawParameters<'a>) -> Self {
        self.draw_params = Some(params);
        self
    }

    pub fn transform(mut self, transform: Mat4<f32>) -> Self {
        self.transform = Some(transform);
        self
    }

    pub fn build(self) -> Object<'a> {
        Object {
            vertex_buffer: self.vertex_buffer,
            indices: self.indices,
            draw_params: self.draw_params.unwrap_or_else(|| Default::default()),
            transform: self.transform.unwrap_or_else(|| nalgebra::new_identity(4)),
            shader_type: ShaderType::Unlit
        }
    }
}

pub struct Object<'a> {
    vertex_buffer: VertexBufferAny,
    indices: IndicesSource<'a>,
    draw_params: DrawParameters<'a>,
    transform: Mat4<f32>,
    shader_type: ShaderType,
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

    pub fn draw<F: Facade, S: Surface>(&mut self, surface: &mut S, facade: &F,
                                       camera: &Camera, obj: &Object) -> Result<(), DrawError> {
        let &mut EngineContext { ref vertex_shader, ref mut shader_map } = self;
        let uniforms = match obj.shader_type {
            ShaderType::Unlit =>
                uniform! {
                    proj_matrix: camera.projection_matrix(),
                    view_matrix: camera.view_matrix(),
                    transform: obj.transform,
                },
        };
        let fragment_shader = Self::get_shader(shader_map, obj.shader_type);
        let program = Program::from_source(facade, vertex_shader,
                                           fragment_shader, None).unwrap();
        surface.draw(&obj.vertex_buffer, obj.indices.clone(),
                     &program, &uniforms, &obj.draw_params)
    }

    fn get_shader(shader_map: &mut HashMap<ShaderType, String>, shader_type: ShaderType) -> &str {
        shader_map.entry(shader_type).or_insert_with(|| {
            let mut shader = String::new();
            File::open(shader_type.to_filename()).unwrap().read_to_string(&mut shader).unwrap();
            shader
        })
    }
}

pub struct Grid<'a> {
    pub parent: Object<'a>,
    dim: u16,
}

impl<'a> Grid<'a> {
    pub fn new<F: Facade>(facade: &F, dim: u16) -> Self {
        let mut shape = Vec::new();
        let len = dim as f32 / 10.;
        for i in 0..dim * 2 + 1 {
            let i = i as f32;
            let v1 = Vertex::new(-len + i * 0.1, -len, 0.);
            let v2 = Vertex::new(-len + i * 0.1, len, 0.);

            let v3 = Vertex::new(-len, -len + i * 0.1, 0.);
            let v4 = Vertex::new(len, -len + i * 0.1, 0.);

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

        let vb = VertexBuffer::new(facade, shape).into_vertex_buffer_any();
        let indices = NoIndices(PrimitiveType::LinesList);
        let parent = ObjectBuilder::new(vb, indices)
            .draw_params(params)
            .build();

        Grid { parent: parent, dim: dim }
    }
}

pub struct Cube<'a> {
    pub parent: Object<'a>,
    dim: f32,
    pos: Vec3<f32>
}

impl<'a> Cube<'a> {
    pub fn new<F: Facade>(facade: &F, dim: f32, pos: Vec3<f32>) -> Self {
        // let image = image::load(::std::io::Cursor::new(&include_bytes!("../resources/cube.png")[..]),
        //                         image::PNG).unwrap();
        // let tex = glium::texture::CompressedSrgbTexture2d::new(&display, image);

        let params = DrawParameters {
            depth_test: DepthTest::IfLess,
            depth_write: true,
            .. Default::default()
        };

        let parent = ObjectBuilder::from_obj(facade, "resources/cube.obj",
                                             NoIndices(PrimitiveType::TrianglesList))
            .draw_params(params)
            .build();

        Cube { parent: parent, dim: dim, pos: pos }
    }

    pub fn get_rotation_mat(t: time::Timespec) -> Mat4<f32> {
        let sec = (t.sec as f64) + ((t.nsec as f64)/1e9);

        Mat4::new(sec.cos() as f32,  -sec.sin() as f32, 0., 0.,
                  sec.sin() as f32,  sec.cos() as f32,  0., 0.,
                  0.,                0.,                1., 0.,
                  0.,                0.,                0., 1.)
    }
}

fn load_obj<F: Facade, R: Read>(facade: &F, data: &mut BufReader<R>) -> VertexBufferAny {
    let data = obj::Obj::load(data);
    let mut vertex_data = Vec::new();

    for shape in data.object_iter().next().unwrap().group_iter().flat_map(|g| g.indices().iter()) {
        match shape {
            &genmesh::Polygon::PolyTri(genmesh::Triangle { x: v1, y: v2, z: v3 }) => {
                for v in [v1, v2, v3].iter() {
                    let position = data.position()[v.0];
                    let texture = v.1.map(|index| data.texture()[index]);
                    //let normal = v.2.map(|index| data.normal()[index]);

                    let texture = texture.unwrap_or([0.0, 0.0]);
                    //let normal = normal.unwrap_or([0.0, 0.0, 0.0]);

                    vertex_data.push(Vertex {
                        position: position,
                        tex_coord: texture,
                    })
                }
            },
            _ => {println!("{:?}", shape); unimplemented!()}
        }
    }

    VertexBuffer::new(facade, vertex_data).into_vertex_buffer_any()
}
