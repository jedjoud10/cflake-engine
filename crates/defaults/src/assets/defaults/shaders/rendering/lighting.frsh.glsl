#version 460 core
#load general
#include "defaults/shaders/rendering/sky.func.glsl"
#include "defaults/shaders/rendering/sun.func.glsl"
#include "defaults/shaders/rendering/shadows.func.glsl"
#include "defaults/shaders/rendering/post.func.glsl"

out vec4 color;
uniform sampler2D diffuse_texture; // 0
uniform sampler2D emissive_texture; // 1
uniform sampler2D normals_texture; // 2
uniform sampler2D position_texture; // 3
uniform sampler2D depth_texture; // 4
uniform sampler2D sky_gradient; // 5
uniform sampler2D shadow_map; // 6
uniform vec3 sunlight_dir;
uniform mat4 lightspace_matrix;
uniform float sunlight_strength;
uniform mat4 inverse_pr_matrix;
uniform mat4 pv_matrix;
uniform vec3 camera_pos;
uniform vec3 camera_dir;
in vec2 uvs;

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
};

// Camera data like position, matrices
struct CameraData {
	vec3 position;
	vec3 direction;
	mat4 pv_matrix;
};

// Scene data like skyboxes and suchs
struct SceneData {
	float time_of_day;
};

#define PI 3.1415926538

// Normal distribution function
// GGX/Trowbridge-reitz model
float ndf(float alpha, vec3 n, vec3 h) {
	float num = pow(alpha, 2.0);
	float prod = max(dot(n, h), 0.0);
	float denom = PI * pow((pow(prod, 2) * (pow(alpha, 2) - 1) + 1), 2);
	denom = max(denom, 0.0001);
	return num / denom;
}



// Schlick/GGX model
float g1(float k, vec3 n, vec3 x) {
	float num = max(dot(n, x), 0);
	float denom = max(num * (1 - k) + k, 0.001);
	return num / denom;
}
// Smith model
float gsf(float roughness, vec3 n, vec3 v, vec3 l) {
	float a = (roughness * roughness);
	float k = a / 2;
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
	float alpha = pow(roughness, 2);
	vec3 num = ndf(alpha, n, h) * gsf(alpha, n, v, l) * fresnel(f0, v, h, n);
	float denom = 4 * max(dot(v, n), 0.0) * max(dot(l, n), 0.0);
	denom = max(denom, 0.0001);
	return num / denom;
}

vec3 brdf(SunData sun, PixelData pixel, CameraData camera, SceneData scene) {
	// Main vectors
	vec3 n = normalize(pixel.normal);
	vec3 v = normalize(camera.position - pixel.position);
	vec3 l = -normalize(sun.direction);
	vec3 h = normalize(v + l);

	// Constants
	float roughness = 0.4;
	float metallic = 0.0;
	vec3 f0 = mix(vec3(0.04), pixel.diffuse, metallic);
	
	// Ks and Kd
	vec3 ks = fresnel(f0, v, h, n);
	vec3 kd = (1 - ks) * (1 - metallic) * 0.0;

	// Le diffuse and specular
	vec3 brdf = kd * (pixel.diffuse / PI) + specular(f0, roughness, v, l, n, h);
	vec3 outgoing = brdf * sun.color * sun.strength * max(dot(l, n), 0.0);

	return specular(f0, roughness, v, l, n, h);
}

// PBR TIMEEE
vec3 shade(SunData sun, PixelData pixel, CameraData camera, SceneData scene) {   
	return brdf(sun, pixel, camera, scene);
}


void main() {
	// Sample the G-Buffer textures
	vec3 normal = normalize(texture(normals_texture, uvs).xyz);
	vec3 diffuse = texture(diffuse_texture, uvs).xyz;
	vec3 emissive = texture(emissive_texture, uvs).xyz;
	vec3 position = texture(position_texture, uvs).xyz;

	// Calculate the dot product using the sun's direction vector and the up vector
	float sun_dot_product = dot(-sunlight_dir, vec3(0, 1, 0));
	float time_of_day = sun_dot_product * 0.5 + 0.5;
	float global_sunlight_strength = calculate_sun_strength(time_of_day) * sunlight_strength;	

	// Le pixel direction
	vec3 pixel_dir = normalize((inverse_pr_matrix * vec4(uvs * 2 - 1, 0, 1)).xyz);

	// Get fragment depth
	vec3 final_color = vec3(0, 0, 0);
	float odepth = texture(depth_texture, uvs).x;

	// Depth test with the sky
	if (odepth == 1.0) {
		// Sky gradient texture moment
		float sky_uv_sampler = dot(pixel_dir, vec3(0, 1, 0));
		final_color = calculate_sky_color(sky_gradient, pixel_dir, sky_uv_sampler, time_of_day);
		final_color += max(pow(dot(pixel_dir, normalize(-sunlight_dir)), 4096), 0) * global_sunlight_strength * 40;
	} else {
		// Shadow map
		//float in_shadow = calculate_shadows(position, normal, sunlight_dir, lightspace_matrix, shadow_map);

		// Construct the structs just cause they look pretty
		SunData sun = SunData(sunlight_dir, global_sunlight_strength, vec3(1));
		PixelData pixel = PixelData(diffuse, normal, emissive, position);
		CameraData camera = CameraData(camera_pos, camera_dir, pv_matrix);
		SceneData scene = SceneData(time_of_day);
		final_color = shade(sun, pixel, camera, scene);
	}

	color = vec4(post_rendering(uvs, final_color), 1.0);
}