// https://github.com/cohaereo/egui_glfw_gl/blob/master/src/painter.rs
#version 460 core

// Window resolution sheize
layout(set = 0, binding = 0) uniform WindowUniform {
    uint width;
    uint height;
} window;

layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_tc;
layout(location = 2) in vec4 a_srgba; // 0-255 sRGB

layout(location = 0) out vec4 v_rgba;
layout(location = 1) out vec2 v_tc;

void main() {
    gl_Position = vec4(
        2.0 * a_pos.x / float(window.width) - 1.0,
        1.0 - 2.0 * a_pos.y / float(window.height),
        0.0,
        1.0);
    v_rgba = a_srgba;
    v_tc = a_tc;
}