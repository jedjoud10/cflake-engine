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

// This will create a simple Plane-Ray intersection test
void intersects(vec3 eye, vec3 ray, float d, out bool hit, out vec3 point) {
    const vec3 n = vec3(0, -1, 0);
    float denom = dot(ray, n);
    if (denom > 0.001) {
        if (eye.y > d) {
            point = vec3(0);
            hit = false;
            return;
        }

        float t = (dot(-eye, n) - d) / denom;
        vec3 p = eye + t * ray;
        point = p;
        hit = true;
    } else {
        hit = false;
        point = vec3(0);
    }
}

void main() {
    // Get the main sky color
    float offset = (sun_dir.y + 1.0) / 2.0;
    vec3 color = texture(gradient, vec2(0.99, m_tex_coord.y)).rgb; 

    // Add the sun as a bright circle
    float size = dot(sun_dir, normalize(m_position)) + ((sun_size - 1) / 90.0);
    float circle = max(pow(size, 15 * sun_intensity), 0); 
    color = mix(color, vec3(1.0), circle);

    frag = color;
    //frag = abs(intersects(camera, normalize(m_normal), 3));
    /*
    bool hit = false;
    vec3 point = vec3(0);
    intersects(camera, -normalize(m_position), 30, hit, point);
    frag = point;
    */
    //frag = normalize(m_position);
}