#version 140

const uint COLOR_TYPE = uint(0);
const uint TEXTURE_RGB_TYPE = uint(1);
const uint TEXTURE_ALPHA_TYPE = uint(2);

in vec3 v_coord;
in vec2 v_tex_coord;

uniform sampler2D tex;
uniform vec3 color;
uniform uint type;

out vec4 out_color;

void main() {
    if (type == COLOR_TYPE) {
        out_color = vec4(color, 1.);
    } else if (type == TEXTURE_RGB_TYPE) {
        out_color = texture(tex, v_tex_coord);
    } else if (type == TEXTURE_ALPHA_TYPE) {
        float a = texture(tex, v_tex_coord).r;
        out_color = vec4(color, a);
    }
}
