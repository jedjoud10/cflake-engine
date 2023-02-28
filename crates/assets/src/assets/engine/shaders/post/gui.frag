// https://github.com/cohaereo/egui_glfw_gl/blob/master/src/painter.rs
#version 460 core
#include "engine/shaders/math/conversions.func.glsl"
uniform sampler2D image;
in vec4 v_rgba;
in vec2 v_tc;
out vec4 f_color;

// TODO: Is there a way to remove all of this for a simpler solution?
void main() {
    // Need to convert from SRGBA to linear.
    vec4 texture_rgba = linear_from_srgba(texture(image, vec2(v_tc.x, v_tc.y)) * 255.0);
    f_color = v_rgba * texture_rgba;
}
