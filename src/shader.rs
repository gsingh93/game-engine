#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum ShaderType {
    UnlitColor,
    UnlitTexture,
}

impl ShaderType {
    pub fn to_filename(&self) -> &'static str {
        match self {
            &ShaderType::UnlitColor => "shaders/unlit.fragment.glsl",
            &ShaderType::UnlitTexture => "shaders/unlit-texture.fragment.glsl"
        }
    }
}
