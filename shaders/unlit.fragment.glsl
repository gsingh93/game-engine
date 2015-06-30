#version 140

in vec3 v_coord;
in vec2 v_tex_coord;

uniform vec3 color;

out vec4 out_color;

void main() {
    out_color = vec4(color, 1.);
}
