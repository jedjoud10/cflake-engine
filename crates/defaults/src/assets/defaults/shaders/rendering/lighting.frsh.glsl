#version 460 core
#load general

out vec4 color;
uniform sampler2D diffuse_texture;
uniform sampler2D emissive_texture;
uniform sampler2D normals_texture;
uniform sampler2D position_texture;
uniform sampler2D mask_texture;
uniform sampler2D depth_texture;
uniform sampler2D shadow_map;
uniform samplerCube skybox;
uniform vec3 sunlight_dir;
uniform mat4 lightspace_matrix;
uniform float sunlight_strength;
uniform mat4 inverse_pr_matrix;
uniform mat4 pv_matrix;
uniform vec3 camera_pos;
uniform vec3 camera_dir;
uniform float time_of_day;
in vec2 uvs;

#include "defaults/shaders/rendering/shadows.func.glsl"
#include "defaults/shaders/rendering/post.func.glsl"

// Sun data that will be passed to the rendering equation
struct SunData {
	vec3 direction;
	float strength;
	vec3 color;
};

// Data fetched from the G-Buffer, and the current pixel direction
struct PixelData {
	// Given from the G-Buffer
	vec3 diffuse;
	vec3 normal;
	vec3 emissive;
	vec3 position;
	float ao;
	float roughness;
	float metallic;

	// Calculated
	float in_shadow;
};

// Camera data like position, matrices
struct CameraData {
	vec3 position;
	vec3 direction;
	mat4 pv_matrix;
};

#define PI 3.1415926538

// Normal distribution function
// GGX/Trowbridge-reitz model
float ndf(float roughness, vec3 n, vec3 h) {
	float a = pow(roughness, 2);
	float a2 = a*a;

	float n_dot_h = max(dot(n, h), 0.0);
	float n_dot_h_2 = pow(n_dot_h, 2);
	
	float num = a2;
	float denom = n_dot_h_2 * (a2 - 1) + 1;
	denom = PI * denom * denom;
	return num / denom;
}

// Schlick/GGX model
float g1(float k, vec3 n, vec3 x) {
	float num = max(dot(n, x), 0);
	float denom = num * (1 - k) + k;
	return num / denom;
}
// Smith model
float gsf(float roughness, vec3 n, vec3 v, vec3 l) {
	float r = (roughness + 1.0);
    float k = (r*r) / 8.0;
	return g1(k, n, v) * g1(k, n, l);
}

// Fresnel function
vec3 fresnel(vec3 f0, vec3 v, vec3 h, vec3 n) {
	float prod = max(dot(v, h), 0);
	float clamped = clamp(1 - prod, 0, 1);
	return f0 + (1 - f0) * pow(clamped, 5);
}

// Cook-torrence model for specular
vec3 specular(vec3 f0, float roughness, vec3 v, vec3 l, vec3 n, vec3 h) {
	vec3 num = ndf(roughness, n, h) * gsf(roughness, n, v, l) * fresnel(f0, v, h, n);
	float denom = 4 * max(dot(v, n), 0.0) * max(dot(l, n), 0.0);
	return num / max(denom, 0.001);
}

// Bidirectional reflectance distribution function, aka PBRRRR
vec3 brdf(SunData sun, PixelData pixel, CameraData camera) {
	// Main vectors
	vec3 n = normalize(pixel.normal);
	vec3 v = normalize(camera.position - pixel.position);
	vec3 l = -normalize(sun.direction);
	vec3 h = normalize(v + l);

	// Constants
	float roughness = max(pixel.roughness, 0.05);
	float metallic = pixel.metallic;
	vec3 f0 = mix(vec3(0.04), pixel.diffuse, metallic);
	
	// Ks and Kd
	vec3 ks = fresnel(f0, v, h, n);
	vec3 kd = (1 - ks) * (1 - metallic);

	// Le diffuse and specular
	vec3 brdf = kd * (pixel.diffuse / PI) + specular(f0, roughness, v, l, n, h);
	vec3 outgoing = pixel.emissive + brdf * sun.color * sun.strength * max(dot(l, n), 0.0);

	return outgoing;
}

// Calculate the shaded color for a single pixel 
vec3 shade(SunData sun, PixelData pixel, CameraData camera) {   
	// The shaded pixel color
	vec3 color = brdf(sun, pixel, camera) * (1 - pixel.in_shadow);
	vec3 v = normalize(camera.position - pixel.position);

	// Ambient color
	color += 0.03 * pixel.diffuse * pixel.ao;
	return color;
}



// Calculate the sun's strength using the sun's dot product
float calculate_sun_strength(float sun_up_factor) {
    return clamp(sun_up_factor * 6 - 2.4, 0, 1);
}

void main() {
	// Sample the G-Buffer textures
	vec3 normal = normalize(texture(normals_texture, uvs).rgb);
	vec3 diffuse = texture(diffuse_texture, uvs).rgb;
	vec3 emissive = texture(emissive_texture, uvs).rgb;
	vec3 position = texture(position_texture, uvs).rgb;
	vec3 mask = texture(mask_texture, uvs).rgb;

	// Calculate the dot product using the sun's direction vector and the up vector
	float global_sunlight_strength = calculate_sun_strength(time_of_day) * sunlight_strength;	

	// Le pixel direction (going from the camera towards the surface)
	vec3 eye_dir = normalize((inverse_pr_matrix * vec4(uvs * 2 - 1, 0, 1)).xyz);

	// Get fragment depth
	vec3 final_color = vec3(0, 0, 0);
	float odepth = texture(depth_texture, uvs).x;

	// Depth test with the sky
	if (odepth == 1.0) {
		// Sky gradient texture moment
		float sky_uv_sampler = dot(eye_dir, vec3(0, 1, 0));
		final_color = texture(skybox, eye_dir).xyz;
		final_color += max(pow(dot(eye_dir, normalize(-sunlight_dir)), 4096), 0) * global_sunlight_strength * 40;
	} else {
		// Shadow map
		float in_shadow = calculate_shadows(position, normal, sunlight_dir, lightspace_matrix, shadow_map);

		// Construct the structs just cause they look pretty
		SunData sun = SunData(sunlight_dir, global_sunlight_strength, vec3(1));
		PixelData pixel = PixelData(diffuse, normal, emissive, position, mask.r, mask.g, mask.b, in_shadow);
		CameraData camera = CameraData(camera_pos, camera_dir, pv_matrix);
		final_color = shade(sun, pixel, camera);
	}

	color = vec4(final_color, 0);
}