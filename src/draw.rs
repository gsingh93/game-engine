use std::borrow::Cow;
use std::io::{BufReader, Cursor, Read};
use std::fs::File;
use std::path::Path;

use shader::{FragmentShaderType, VertexShaderType};
use camera::Camera;

use freetype as ft;

use genmesh;

use glium::{BlendingFunction, DepthTest, Display, DrawParameters, LinearBlendingFactor,
            VertexBuffer};
use glium::backend::Facade;
use glium::index::{IndicesSource, NoIndices, PrimitiveType};
use glium::texture::{ClientFormat, RawImage2d, Texture2d};
use glium::uniforms::{MinifySamplerFilter, MagnifySamplerFilter, SamplerBehavior,
                      SamplerWrapFunction, UniformValue, Uniforms};
use glium::vertex::VertexBufferAny;

use image;

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
    vertex_buffer: VertexBufferAny,
    indices: IndicesSource<'a>,
    draw_params: Option<DrawParameters<'a>>,
    transform: Option<Mat4<f32>>,
    vert_shader_type: Option<VertexShaderType>,
    frag_shader_type: Option<FragmentShaderType>,
}

impl<'a> ObjectBuilder<'a> {
    pub fn new<I: Into<IndicesSource<'a>>>(vb: VertexBufferAny, indices: I) -> Self {
        ObjectBuilder {
            vertex_buffer: vb,
            indices: indices.into(),
            draw_params: None,
            transform: None,
            vert_shader_type: None,
            frag_shader_type: None,
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
    pub vertex_buffer: VertexBufferAny,
    pub indices: IndicesSource<'a>,
    pub draw_params: DrawParameters<'a>,
    pub transform: Mat4<f32>,
    pub vert_shader_type: VertexShaderType,
    pub frag_shader_type: FragmentShaderType,
}

pub trait GameObject {
    fn parent(&self) -> &Object;
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
            ("transform", UniformValue::Mat4(*self.parent().transform.as_array())),
            ("color", UniformValue::Vec3([1., 1., 1.]))])
    }
}

impl<'a> Grid<'a> {
    pub fn new<F: Facade>(facade: &F, dim: u16) -> Self {
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

        let vb = VertexBuffer::new(facade, shape).into_vertex_buffer_any();
        let indices = NoIndices(PrimitiveType::LinesList);
        let parent = ObjectBuilder::new(vb, indices)
            .draw_params(params)
            .build();

        Grid { parent: parent }
    }
}

pub struct Cube<'a> {
    parent: Object<'a>,
    texture: Texture2d,
}

impl<'a> GameObject for Cube<'a> {
    fn parent(&self) -> &Object {
        &self.parent
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
            ("transform", UniformValue::Mat4(*self.parent().transform.as_array())),
            ("tex", UniformValue::Texture2d(&self.texture, Some(sampler)))])
    }
}

impl<'a> Cube<'a> {
    pub fn new<F: Facade>(facade: &F, dim: f32, pos: Vec3<f32>) -> Self {
        let image = image::load(Cursor::new(&include_bytes!("../resources/cube.png")[..]),
                                image::PNG).unwrap();
        let tex = Texture2d::new(facade, image);

        let params = DrawParameters {
            depth_test: DepthTest::IfLess,
            depth_write: true,
            .. Default::default()
        };

        let mut transform: Mat4<f32> = nalgebra::new_identity(4);
        transform = transform * dim;
        transform.set_col(3, Vec4::new(pos.x, pos.y, pos.z, 1.));

        let parent = ObjectBuilder::from_obj(facade, "resources/cube.obj",
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
    chars: Vec<Char<'a>>
}

impl<'a> Text<'a> {
    pub fn new(display: &Display, x: f32, y: f32, text: &str) -> Self {
        let (w, h) = ::get_display_dim(display);
        let (sx, sy) = (2. / w as f32, 2. / h as f32);

        let freetype = ft::Library::init().unwrap();
        let face = freetype.new_face("resources/FiraSans-Regular.ttf", 0).unwrap();
        face.set_pixel_sizes(0, 16).unwrap();

        let mut x = x;
        let mut y = y;
        let mut chars = Vec::new();
        for c in text.chars() {
            face.load_char(c as usize, ft::face::RENDER).unwrap();
            let g = face.glyph();

            let bitmap = g.bitmap();
            let texture = Texture2d::new(display, RawImage2d {
                data: Cow::Borrowed(bitmap.buffer()),
                width: bitmap.width() as u32, height: bitmap.rows() as u32,
                format: ClientFormat::U8
            });
            chars.push(Char::new(display,
                                 x, y,
                                 g.bitmap_left() as f32, g.bitmap_top() as f32,
                                 bitmap.width() as f32, bitmap.rows() as f32,
                                 sx, sy,
                                 texture));

            x += (g.advance().x >> 6) as f32 * sx;
            y += (g.advance().y >> 6) as f32 * sy;
        }

        Text { chars: chars }
    }

    pub fn into_chars(self) -> Vec<Char<'a>> {
        self.chars
    }
}

pub struct Char<'a> {
    parent: Object<'a>,
    texture: Texture2d
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
            ("transform", UniformValue::Mat4(*self.parent().transform.as_array())),
            ("color", UniformValue::Vec3([0., 1., 0.])),
            ("tex", UniformValue::Texture2d(&self.texture, Some(sampler)))])
    }
}

impl<'a> Char<'a> {
    fn new(display: &Display, x: f32, y: f32, left: f32, _: f32, width: f32, height: f32,
           sx: f32, sy: f32, texture: Texture2d) -> Self {
        let x = x + left * sx;
        let y = y;// - top * sy; // FIXME
        let width = width * sx;
        let height = height * sy;

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

        let parent = ObjectBuilder::new(vb, NoIndices(PrimitiveType::TrianglesList))
            .draw_params(params)
            .vert_shader(VertexShaderType::Gui)
            .build();
        Char { parent: parent, texture: texture }
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
