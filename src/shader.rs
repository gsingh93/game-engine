#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum ShaderType {
    Unlit
}

impl ShaderType {
    pub fn to_filename(&self) -> &'static str {
        match self {
            &ShaderType::Unlit => "shaders/unlit.fragment.glsl"
        }
    }
}
