#version 140

in vec3 position;
in vec2 tex_coord;

uniform mat4 proj_matrix;
uniform mat4 view_matrix;
uniform mat4 transform;

out vec3 v_coord;
out vec2 v_tex_coord;

void main() {
    v_coord = position;
    v_tex_coord = tex_coord;
    gl_Position = proj_matrix * view_matrix * transform * vec4(position, 1.);
}
