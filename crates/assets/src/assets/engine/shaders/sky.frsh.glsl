#version 460 core
out vec4 frag;

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

in vec2 m_tex_coord;
in vec3 m_position;
in vec3 m_normal;

void main() {
    // Get the main sky color
    float offset = (sun_dir.y + 1.0) / 2.0;
    vec3 color = texture(gradient, vec2(offset, m_tex_coord.y)).rgb; 

    // Add the sun as a bright circle
    float size = dot(sun_dir, normalize(m_position)) + ((sun_size - 1) / 90.0);
    float circle = max(pow(size, 512 * sun_intensity), 0); 
    color = mix(color, vec3(1.0), circle);

    frag = vec4(color, 1.0);
}