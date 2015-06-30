#version 140

in vec3 v_coord;
in vec2 v_tex_coord;

uniform sampler2D tex;

out vec4 color;

void main() {
    color = texture(tex, v_tex_coord);
}
