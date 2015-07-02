use std::fmt::Debug;
use std::hash::Hash;

// TODO: Make this As<Path>?
pub trait ShaderType : Copy + Clone + Debug + Eq + Hash + PartialEq {
    fn to_filename(&self) -> &'static str;
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum FragmentShaderType {
    Unlit,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum VertexShaderType {
    Perspective,
    Gui,
}

impl ShaderType for FragmentShaderType {
    fn to_filename(&self) -> &'static str {
        match self {
            &FragmentShaderType::Unlit => "shaders/unlit.fragment.glsl",
        }
    }
}

impl ShaderType for VertexShaderType {
    fn to_filename(&self) -> &'static str {
        match self {
            &VertexShaderType::Perspective => "shaders/perspective.vertex.glsl",
            &VertexShaderType::Gui => "shaders/gui.vertex.glsl"
        }
    }
}
