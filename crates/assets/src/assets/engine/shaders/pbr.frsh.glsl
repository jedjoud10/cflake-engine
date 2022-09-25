#version 460 core
#include "engine/shaders/models.func.glsl"
#include "engine/shaders/shadow.func.glsl"
#include "engine/shaders/clustered.func.glsl"
out vec3 frag;

// Main PBR uniforms
uniform float roughness;
uniform float bumpiness;
uniform float metallic;
uniform float ambient_occlusion;
uniform vec3 tint;
uniform sampler2D albedo;
uniform sampler2D normal;
uniform sampler2D mask;
uniform vec2 scale;

// Uniforms coming from the camera
uniform vec3 camera;
uniform vec3 camera_forward;

// Uniforms set by the main scene
uniform vec3 light_dir;
uniform vec3 light_color;
uniform float light_strength;

// Directional shadow mapping
uniform sampler2DShadow shadow_map;
uniform mat4 shadow_lightspace_matrix;

// Clustered shading data
uniform uint cluster_size;
uniform uvec2 resolution;

// Point lights that are in the scene
uniform uint point_lights_num;
layout(std430) readonly buffer point_lights
{
    PackedPointLight lights[];
};

// Clustered shading clusters
// Clustered shading indices

// Data given by the vertex shader
in vec3 m_position;
in vec3 m_normal;
in vec3 m_tangent;
in vec3 m_bitangent;
in vec3 m_color;
in vec2 m_tex_coord;

// Light data struct
struct LightData {
	vec3 backward;
	vec3 color;
	float strength;
};

// Camera data struct
struct CameraData {
	vec3 view;
	vec3 half_view;
	vec3 position;
};

// Surface data struct 
struct SurfaceData {
	vec3 diffuse;
	vec3 mask;
	vec3 normal;
	vec3 position;
};

// Bidirectional reflectance distribution function, aka PBRRRR
vec3 brdf(SurfaceData surface, CameraData camera, LightData light) {
	float roughness = max(surface.mask.g, 0.06);
	float metallic = surface.mask.b;
	float visibility = surface.mask.r;
	vec3 f0 = mix(vec3(0.04), surface.diffuse, metallic);
	
	// Ks and Kd
	vec3 ks = fresnel(f0, camera.view, camera.half_view, surface.normal);
	vec3 kd = (1 - ks) * (1 - metallic);

	/*
	// Check if the fragment is in shadow
	float shadow = is_in_shadow(
		surface.position,
		sun.backward,
		shadow_lightspace_matrix,
		shadow_map
	);
	*/
	float shadow = 0.0;

	// Calculate diffuse and specular
	vec3 brdf = kd * (surface.diffuse / PI) + specular(f0, roughness, camera.view, light.backward, surface.normal, camera.half_view);
	brdf = brdf * light.color * light.strength * max(dot(light.backward, surface.normal), 0.0) * (1 - shadow);
	return brdf;
}

void main() {
	// Fetch the textures and their texels
    vec3 diffuse = texture(albedo, m_tex_coord * scale).xyz * tint;
	vec3 bumps = texture(normal, m_tex_coord * scale).xyz * 2.0 - 1.0;
	vec3 mask = texture(mask, m_tex_coord * scale).xyz * vec3(1 / ambient_occlusion, roughness, metallic);

    // Calculate the normal mapped bumpiness
	bumps.xy *= bumpiness;

	// Calculate the world space normals
	mat3 tbn = mat3(
		normalize(m_tangent),
		normalize(m_bitangent),
		normalize(m_normal));
	vec3 normal = normalize(tbn * normalize(bumps));

	// Iterate through all the point light
	vec3 sum = vec3(0, 0, 0);
	for(int i = 0; i < point_lights_num; i++) {
		vec3 light_position = lights[i].position_attenuation.xyz;
		vec3 dir = normalize(light_position - m_position);
        float dist = length(dir);
        float attenuation = 1.0 / ((dist * dist) * lights[i].position_attenuation.w);
		vec3 radiance = lights[i].color.xyz * attenuation;

		LightData sun = LightData(dir, radiance, 1.0);
		SurfaceData surface = SurfaceData(diffuse, mask, normal, m_position);
		vec3 view = normalize(camera - m_position);
		CameraData camera = CameraData(view, normalize(view + dir), camera);
		sum += brdf(surface, camera, sun);
	}

	// Add the ambient lighting
	sum += 0.03 * diffuse * mask.r;

	// Color of the final result
	frag = sum;
	//frag = brdf(surface, camera, sun);
	//frag = vec3(dist / 10.0);
}