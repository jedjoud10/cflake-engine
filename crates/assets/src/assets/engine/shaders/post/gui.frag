// https://github.com/cohaereo/egui_glfw_gl/blob/master/src/painter.rs
#version 460 core
//uniform sampler2D image;
layout(location = 0) in vec4 v_rgba;
layout(location = 1) in vec2 v_tc;
layout(location = 0) out vec4 f_color;

void main() {
    // Need to convert from SRGBA to linear.
    //vec4 texture_rgba = linear_from_srgba(texture(image, vec2(v_tc.x, v_tc.y)) * 255.0);
    //f_color = v_rgba * texture_rgba;
    f_color = vec4(1);
}
