#version 460 core
out vec4 frag;

// Main sky texture
uniform sampler2D gradient;
uniform float offset;
uniform float time;

// Sun params
uniform float sun_intensity;
uniform float sun_radius;
uniform vec3 sun_dir;

// Cloud params
uniform float cloud_speed;
uniform float cloud_coverage;

in vec2 m_tex_coord_0;

void main() {
    frag = texture(gradient, vec2(0.0, m_tex_coord_0.y));
}