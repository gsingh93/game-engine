use std::io::{BufReader, Read};
use std::fs::File;
use std::path::Path;
use std::rc::Rc;

use {Character, EngineContext};
use shader::{FragmentShaderType, VertexShaderType};
use camera::Camera;

use freetype as ft;

use genmesh;

use glium::{BlendingFunction, DepthTest, Display, DrawParameters, LinearBlendingFactor,
            VertexBuffer};
use glium::backend::Facade;
use glium::index::{IndicesSource, NoIndices, PrimitiveType};
use glium::texture::Texture2d;
use glium::uniforms::{MinifySamplerFilter, MagnifySamplerFilter, SamplerBehavior,
                      SamplerWrapFunction, UniformValue, Uniforms};
use glium::vertex::VertexBufferAny;

use nalgebra::{self, Col, Mat4, Vec3, Vec4};

use obj;

use time;

const COLOR_TYPE: u32 = 0;
const TEXTURE_RGB_TYPE: u32 = 1;
const TEXTURE_ALPHA_TYPE: u32 = 2;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
}

impl Vertex {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vertex { position: [x, y, z], tex_coord: [0., 0.] }
    }

    fn with_texture(x: f32, y: f32, z: f32, u: f32, v: f32) -> Self {
        Vertex { position: [x, y, z], tex_coord: [u, v] }
    }

}

implement_vertex!(Vertex, position, tex_coord);

pub struct ObjectBuilder<'a> {
    vertex_buffer: Option<VertexBufferAny>,
    indices: Option<IndicesSource<'a>>,
    draw_params: Option<DrawParameters<'a>>,
    transform: Option<Mat4<f32>>,
    vert_shader_type: Option<VertexShaderType>,
    frag_shader_type: Option<FragmentShaderType>,
}

impl<'a> ObjectBuilder<'a> {
    pub fn new() -> Self {
        ObjectBuilder {
            vertex_buffer: None,
            indices: None,
            draw_params: None,
            transform: None,
            vert_shader_type: None,
            frag_shader_type: None,
        }
    }

    pub fn vertex_buffer<I: Into<IndicesSource<'a>>>(mut self, vb: VertexBufferAny,
                                                     indices: I) -> Self {
        self.vertex_buffer = Some(vb);
        self.indices = Some(indices.into());
        self
    }

    pub fn from_obj<F, I, P>(facade: &F, path: P, indices: I) -> Self
    where F: Facade, I: Into<IndicesSource<'a>>, P: AsRef<Path> {
        let vb = load_obj(facade, &mut BufReader::new(File::open(path).unwrap()));
        ObjectBuilder::new().vertex_buffer(vb, indices)
    }

    pub fn draw_params(mut self, params: DrawParameters<'a>) -> Self {
        self.draw_params = Some(params);
        self
    }

    pub fn transform(mut self, transform: Mat4<f32>) -> Self {
        self.transform = Some(transform);
        self
    }

    pub fn vert_shader(mut self, vert_shader_type: VertexShaderType) -> Self {
        self.vert_shader_type = Some(vert_shader_type);
        self
    }

    pub fn frag_shader(mut self, frag_shader_type: FragmentShaderType) -> Self {
        self.frag_shader_type = Some(frag_shader_type);
        self
    }

    pub fn build(self) -> Object<'a> {
        Object {
            name: None,
            vertex_buffer: self.vertex_buffer,
            indices: self.indices,
            draw_params: self.draw_params.unwrap_or_else(|| Default::default()),
            transform: self.transform.unwrap_or_else(|| nalgebra::new_identity(4)),
            vert_shader_type: self.vert_shader_type.unwrap_or(VertexShaderType::Perspective),
            frag_shader_type: self.frag_shader_type.unwrap_or(FragmentShaderType::Unlit),
        }
    }
}

// FIXME: Use getters instead of public fields
pub struct Object<'a> {
    pub name: Option<String>,
    pub vertex_buffer: Option<VertexBufferAny>,
    pub indices: Option<IndicesSource<'a>>,
    pub draw_params: DrawParameters<'a>,
    pub transform: Mat4<f32>,
    pub vert_shader_type: VertexShaderType,
    pub frag_shader_type: FragmentShaderType,
}

pub trait GameObject {
    fn name(&self) -> Option<&str> {
        self.parent().name.as_ref().map(|s| &*s as &str)
    }
    fn update(&mut self) {}
    fn parent(&self) -> &Object;
    fn children(&self) -> Option<&[Box<GameObject>]> {
        None
    }
    fn construct_uniforms(&self, &Camera) -> UniformsVec;
}

struct UniformsVec<'a>(Vec<(&'static str, UniformValue<'a>)>);
impl<'b> Uniforms for UniformsVec<'b> {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
        for v in self.0.iter() {
            f(&v.0, v.1);
        }
    }
}

pub struct Grid<'a> {
    parent: Object<'a>,
}

impl<'a> GameObject for Grid<'a> {
    fn parent(&self) -> &Object {
        &self.parent
    }

    fn construct_uniforms(&self, camera: &Camera) -> UniformsVec {
        UniformsVec(vec![
            ("type", UniformValue::UnsignedInt(COLOR_TYPE)),
            ("proj_matrix", UniformValue::Mat4(*camera.projection_matrix().as_array())),
            ("view_matrix", UniformValue::Mat4(*camera.view_matrix().as_array())),
            ("transform", UniformValue::Mat4(*self.parent.transform.as_array())),
            ("color", UniformValue::Vec3([1., 1., 1.]))])
    }
}

impl<'a> Grid<'a> {
    pub fn new(display: &Display, dim: u16) -> Self {
        let mut shape = Vec::new();
        let len = dim as f32;
        for i in 0..dim * 2 + 1 {
            let i = i as f32;
            let v1 = Vertex::new(-len + i, -len, 0.);
            let v2 = Vertex::new(-len + i, len, 0.);

            let v3 = Vertex::new(-len, -len + i, 0.);
            let v4 = Vertex::new(len, -len + i, 0.);

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

        let vb = VertexBuffer::new(display, shape).into_vertex_buffer_any();
        let indices = NoIndices(PrimitiveType::LinesList);
        let parent = ObjectBuilder::new().vertex_buffer(vb, indices)
            .draw_params(params)
            .build();

        Grid { parent: parent }
    }
}

pub struct Cube<'a> {
    parent: Object<'a>,
    texture: Rc<Texture2d>,
}

impl<'a> GameObject for Cube<'a> {
    fn parent(&self) -> &Object {
        &self.parent
    }

    fn update(&mut self) {
        let mut rot_mat = Self::get_rotation_mat(time::get_time());
        rot_mat.set_col(3, self.parent.transform.col(3));
        self.parent.transform = rot_mat;
    }

    fn construct_uniforms(&self, camera: &Camera) -> UniformsVec {
        let sampler = SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            .. Default::default()
        };
        UniformsVec(vec![
            ("type", UniformValue::UnsignedInt(TEXTURE_RGB_TYPE)),
            ("proj_matrix", UniformValue::Mat4(*camera.projection_matrix().as_array())),
            ("view_matrix", UniformValue::Mat4(*camera.view_matrix().as_array())),
            ("transform", UniformValue::Mat4(*self.parent.transform.as_array())),
            ("tex", UniformValue::Texture2d(&self.texture, Some(sampler)))])
    }
}

impl<'a> Cube<'a> {
    pub fn new(ctxt: &mut EngineContext, dim: f32, pos: Vec3<f32>) -> Self {
        let mut path = ctxt.resource_dir.clone();
        path.push("cube.png");
        let tex = ctxt.texture_cache.get_texture(&ctxt.display, path);

        let params = DrawParameters {
            depth_test: DepthTest::IfLess,
            depth_write: true,
            .. Default::default()
        };

        let mut transform: Mat4<f32> = nalgebra::new_identity(4);
        transform = transform * dim;
        transform.set_col(3, Vec4::new(pos.x, pos.y, pos.z, 1.));

        let mut path = ctxt.resource_dir.clone();
        path.push("cube.obj");
        let parent = ObjectBuilder::from_obj(&ctxt.display, path,
                                             NoIndices(PrimitiveType::TrianglesList))
            .draw_params(params)
            .transform(transform)
            .build();

        Cube { parent: parent, texture: tex }
    }

    pub fn get_rotation_mat(t: time::Timespec) -> Mat4<f32> {
        let sec = (t.sec as f64) + ((t.nsec as f64)/1e9);

        Mat4::new(sec.cos() as f32,  -sec.sin() as f32, 0., 0.,
                  sec.sin() as f32,  sec.cos() as f32,  0., 0.,
                  0.,                0.,                1., 0.,
                  0.,                0.,                0., 1.)
    }
}

pub struct Text<'a> {
    pub parent: Object<'a>,
    chars: Vec<Box<GameObject>>,
    face: ft::Face<'a>, // TODO: Lifetime?
    x: f32,
    y: f32,
}

impl<'a> GameObject for Text<'a> {
    fn parent(&self) -> &Object {
        &self.parent
    }

    fn children(&self) -> Option<&[Box<GameObject>]> {
        Some(&*self.chars)
    }

    fn construct_uniforms(&self, _: &Camera) -> UniformsVec {
        unimplemented!()
    }
}

impl<'a> Text<'a> {
    pub fn new(ctxt: &mut EngineContext, x_start: f32, y_start: f32, text: &str) -> Self {
        let mut path = ctxt.resource_dir.clone();
        path.push("FiraSans-Regular.ttf");

        let freetype = ft::Library::init().unwrap();
        let face = freetype.new_face(path, 0).unwrap();
        face.set_pixel_sizes(0, 16).unwrap();

        // FIXME: This doesn't update after rescaling
        let (w, h) = ::get_display_dim(&ctxt.display);
        let (sx, sy) = (2. / w as f32, 2. / h as f32);

        let mut x = x_start;
        let mut y = y_start;
        let mut chars = Vec::new();
        for c in text.chars() {
            let char = ctxt.texture_cache.get_glyph(&ctxt.display, &face, c);
            let advance_x = char.advance_x * sx;
            let advance_y = char.advance_y * sy;

            chars.push(Box::new(Char::new(&ctxt.display, x, y, sx, sy, char)) as Box<GameObject>);

            x += advance_x;
            y += advance_y;
        }

        Text { chars: chars, face: face, x: x_start, y: y_start,
               parent: ObjectBuilder::new().build() }
    }

    pub fn set_text(&mut self, ctxt: &mut EngineContext, text: &str) {
        let mut path = ctxt.resource_dir.clone();
        path.push("FiraSans-Regular.ttf");

        // FIXME: This doesn't update after rescaling
        let (w, h) = ::get_display_dim(&ctxt.display);
        let (sx, sy) = (2. / w as f32, 2. / h as f32);

        let mut x = self.x;
        let mut y = self.y;
        let mut chars = Vec::new();
        for c in text.chars() {
            let char = ctxt.texture_cache.get_glyph(&ctxt.display, &self.face, c);
            let advance_x = char.advance_x * sx;
            let advance_y = char.advance_y * sy;

            chars.push(Box::new(Char::new(&ctxt.display, x, y, sx, sy, char)) as Box<GameObject>);

            x += advance_x;
            y += advance_y;
        }
        self.chars = chars;
    }
}

pub struct Char<'a> {
    parent: Object<'a>,
    char: Rc<Character>,
}

impl<'a> GameObject for Char<'a> {
    fn parent(&self) -> &Object {
        &self.parent
    }

    fn construct_uniforms(&self, camera: &Camera) -> UniformsVec {
        let clamp = SamplerWrapFunction::Clamp;
        let sampler = SamplerBehavior {
            wrap_function: (clamp, clamp, clamp),
            .. Default::default()
        };
        UniformsVec(vec![
            ("type", UniformValue::UnsignedInt(TEXTURE_ALPHA_TYPE)),
            ("proj_matrix", UniformValue::Mat4(*camera.projection_matrix().as_array())),
            ("view_matrix", UniformValue::Mat4(*camera.view_matrix().as_array())),
            ("transform", UniformValue::Mat4(*self.parent.transform.as_array())),
            ("color", UniformValue::Vec3([0., 1., 0.])),
            ("tex", UniformValue::Texture2d(&self.char.texture, Some(sampler)))])
    }
}

impl<'a> Char<'a> {
    fn new(display: &Display, x: f32, y: f32, sx: f32, sy: f32, char: Rc<Character>) -> Self {
        let x = x + char.left * sx;
        let y = y - (char.height - char.top) * sy;
        let width = char.width * sx;
        let height = char.height * sy;

        // FIXME: Properly handle pitch
        // TODO: What is the correct z value?
        let v1 = Vertex::with_texture(x, y, -0.9, 0., 1.);
        let v2 = Vertex::with_texture(x, y + height, -0.9, 0., 0.);
        let v3 = Vertex::with_texture(x + width, y, -0.9, 1., 1.);
        let v4 = Vertex::with_texture(x + width, y + height, -0.9, 1., 0.);

        let shape = vec![v1, v2, v3, v2, v3, v4];
        let vb = VertexBuffer::new(display, shape).into_vertex_buffer_any();

        let params = DrawParameters {
            // FIXME: This messes with the alpha blending
            // depth_test: DepthTest::IfLess,
            // depth_write: true,
            blending_function: Some(BlendingFunction::Addition {
                source: LinearBlendingFactor::SourceAlpha,
                destination: LinearBlendingFactor::OneMinusSourceAlpha
            }),
            .. Default::default()
        };

        let parent = ObjectBuilder::new()
            .vertex_buffer(vb, NoIndices(PrimitiveType::TrianglesList))
            .draw_params(params)
            .vert_shader(VertexShaderType::Gui)
            .build();
        Char { parent: parent, char: char }
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
