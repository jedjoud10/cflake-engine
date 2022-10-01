// https://github.com/cohaereo/egui_glfw_gl/blob/master/src/painter.rs
#version 460 core
#include "engine/shaders/math/conversions.func.glsl"
uniform vec2 resolution;
layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_tc;
layout(location = 2) in vec4 a_srgba; // 0-255 sRGB
out vec4 v_rgba;
out vec2 v_tc;

// TODO: Is there a way to remove all of this for a simpler solution?
void main() {
    gl_Position = vec4(
        2.0 * a_pos.x / resolution.x - 1.0,
        1.0 - 2.0 * a_pos.y / resolution.y,
        0.0,
        1.0);
    v_rgba = linear_from_srgba(a_srgba);
    v_tc = a_tc;
}