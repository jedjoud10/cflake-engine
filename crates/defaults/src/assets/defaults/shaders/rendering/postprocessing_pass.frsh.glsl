#version 460 core
#load general
#include "defaults/shaders/rendering/screen_space_reflections.func.glsl"
out vec3 color;
in vec2 uvs;
uniform sampler2D color_texture; // 0
uniform sampler2D normals_texture; // 1
uniform sampler2D position_texture; // 2
uniform sampler2D depth_texture; // 3
uniform mat4 pv_matrix;
uniform mat4 pr_matrix;
uniform vec2 nf_planes;

void main() {
    // Sample the textures
	vec3 normal = normalize(texture(normals_texture, uvs).xyz);
	vec3 position = texture(position_texture, uvs).xyz;
    vec3 sampled_color = texture(color_texture, uvs).xyz;
    vec3 pixel_dir = normalize((inverse(pr_matrix) * vec4(uvs * 2 - 1, 0, 1)).xyz);
    float odepth = texture(depth_texture, uvs).x;

    // A vignette effect
    float vignette_strength_x = pow(abs(uvs.x - 0.5), 4);
    float vignette_strength_y = pow(abs(uvs.y - 0.5), 4);
    float vignette_strength = (vignette_strength_x + vignette_strength_y) * 6.0; 
    color = sampled_color * ((1-vignette_strength));
}