// https://github.com/cohaereo/egui_glfw_gl/blob/master/src/painter.rs
#version 460 core
layout(location = 0) in vec4 v_rgba;
layout(location = 1) in vec2 v_tc;
layout(location = 0) out vec4 f_color;

// Font texture 
layout(set = 0, binding = 1) uniform texture2D font;
layout(set = 0, binding = 2) uniform sampler font_sampler;

void main() {
    vec4 color = texture(sampler2D(font, font_sampler), vec2(v_tc.x, v_tc.y));
    f_color = color * v_rgba;
}
