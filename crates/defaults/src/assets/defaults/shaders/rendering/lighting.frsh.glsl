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
	
	// Calculated on the spot
	vec3 direction;
	vec2 uvs;
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
vec3 ndf(float roughness, vec3 normal, vec3 halfway) {
	float alpha = pow(roughness, 2);
	float prod = max(dot(normal, halfway), 0.0001);
	float denum = PI * pow((pow(prod, 2) * (pow(alpha, 2) - 1) + 1), 2);
	return alpha / max(prod, 0.0001);
}



// Geometry shadowing function
// Smith model
vec3 gsf(float roughness, vec3 normal, vec3 view, vec3 light) {
	vec3 g1(float roughness, vec3 normal, vec3 x) {
		float alpha = pow(roughness, 2);
		float k = alpha / 2;
		float prod = max(dot(normal, x), 0.0001);
		float denum = prod * (1 - l) + k;
		return prod / max(denum, 0.0001);
	}
	return g1(roughness, normal, light) * g1(roughness, normal, view);
}

// Fresnel function
float fresnel(vec3 view, float reflectivity, vec3 halfway) {
	float prod = max(dot(view, halfway), 0.0001);
	return reflectivity + (1 - reflectivity) * pow((1 - prod), 5);
}


// Lambertian model for diffuse
vec3 diffuse(vec3 color) {
	return color / PI; 
}

// Cook-torrence model for specular
vec3 specular(vec3 v, vec3 l, vec3 n) {

}

vec3 brdf(SunData sun, PixelData pixel, CameraData camera, SceneData scene) {
	vec3 halfway = (pixel.direction - sun.direction) / 2;
	float ks = fresnel(pixel.direction, 0.2, halfway);
	float kd = 1 - ks;



	return vec3(ks);
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
		PixelData pixel = PixelData(diffuse, normal, emissive, position, pixel_dir, uvs);
		CameraData camera = CameraData(camera_pos, camera_dir, pv_matrix);
		SceneData scene = SceneData(time_of_day);
		final_color = shade(sun, pixel, camera, scene);
	}

	color = vec4(post_rendering(uvs, final_color), 1.0);
}