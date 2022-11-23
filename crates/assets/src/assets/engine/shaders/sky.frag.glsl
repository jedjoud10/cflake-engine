#version 460 core
out vec3 frag;

// Main sky texture
uniform samplerCube cubemap;
uniform float offset;
uniform float time_since_startup;

// Sun params
uniform float sun_intensity;
uniform float sun_size;
uniform vec3 sun_dir;

// Scene params
uniform vec3 camera;

// Given from the last vertex shader
in vec2 m_tex_coord;
in vec3 m_position;

void main() {
    frag = vec4(1.0);
}