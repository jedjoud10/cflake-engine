#version 460 core
layout(location = 0) out vec3 color;
uniform samplerCube cubemap;
in vec3 l_position;

void main() {
    // Create the resulting variables
    vec3 dir = normalize(l_position);
    vec3 irradiance = vec3(0.0);
    vec3 up = vec3(0.0, 1.0, 0.0);
    vec3 right = normalize(cross(up, dir));
    up = normalize(cross(dir, right));

    // Used for convolution
    float sample_delta = 0.025;
    float samples = 0.0;
    const float PI = 3.14159265359;

    // Loop through a sphere using specific delta steps
    for(float phi = 0.0; phi < 2.0 * PI; phi += sample_delta) {
        for(float theta = 0.0; theta < 0.5 * PI; theta += sample_delta) {
            // spherical to cartesian (in tangent space) (I don't know why or how this works ok)
            vec3 sampled_tangent = vec3(sin(theta) * cos(phi),  sin(theta) * sin(phi), cos(theta));
            
            // tangent space to world (ok I see you my boi)
            vec3 sampled_dir = sampled_tangent.x * right + sampled_tangent.y * up + sampled_tangent.z * dir; 

            irradiance += texture(cubemap, sampled_dir).rgb * cos(theta) * sin(theta);
            samples += 1.0;
        }
    }

    // Return the irradiance diffuse lighting
    color = PI * irradiance * (1.0 / samples);
}