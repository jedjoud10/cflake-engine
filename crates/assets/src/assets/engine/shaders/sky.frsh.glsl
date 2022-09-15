#version 460 core
out vec3 frag;

// Main sky texture
uniform sampler2D gradient;
uniform float offset;
uniform float time_since_startup;

// Sun params
uniform float sun_intensity;
uniform float sun_size;
uniform vec3 sun_dir;

// Cloud params
uniform float cloud_speed;
uniform float cloud_coverage;

// Heheheha
uniform vec3 camera;
in vec2 m_tex_coord;
in vec3 m_position;


void main() {
    // Get the main sky color
    float offset = (sun_dir.y + 1.0) / 2.0;
    vec3 color = texture(gradient, vec2(0.99, m_tex_coord.y)).rgb; 

    // Add the sun as a bright circle
    float size = dot(sun_dir, normalize(m_position)) + ((sun_size - 1) / 90.0);
    float circle = max(pow(size, 15 * sun_intensity), 0); 
    color = mix(color, vec3(1.0), circle);
    frag = color;
}