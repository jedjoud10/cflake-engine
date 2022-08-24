#version 460 core
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

// Data given by the vertex shader
in vec3 m_position;
in vec3 m_normal;
in vec3 m_tangent;
in vec3 m_bitangent;
in vec3 m_color;
in vec2 m_tex_coord;

#define PI 3.1415926538

// Sun data struct
struct SunData {
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
vec3 brdf(SurfaceData surface, CameraData camera, SunData sun) {
	// Constants
	float roughness = max(surface.mask.r, 0.05);
	float metallic = pow(surface.mask.g, 5);
	float visibility = min(surface.mask.b, 1.0);
	vec3 f0 = mix(vec3(0.04), surface.diffuse, metallic);
	
	// Ks and Kd
	vec3 ks = fresnel(f0, camera.view, camera.half_view, surface.normal);
	vec3 kd = (1 - ks) * (1 - metallic);

	// Calculate diffuse and specular
	vec3 brdf = kd * (surface.diffuse / PI) + specular(f0, roughness, camera.view, sun.backward, surface.normal, camera.half_view);
	vec3 outgoing = brdf * sun.color * sun.strength * max(dot(sun.backward, surface.normal), 0.0);
	outgoing += 0.03 * surface.diffuse * visibility;
	return outgoing;
}

void main() {
	// Fetch the textures and their texels
    vec3 diffuse = texture(albedo, m_tex_coord * scale).xyz;
	vec3 bumps = texture(normal, m_tex_coord * scale).xyz * 2.0 - 1.0;
	vec3 mask = texture(mask, m_tex_coord * scale).xyz * vec3(roughness, metallic, 1 / ambient_occlusion);

    // Calculate the normal mapped bumpiness
	bumps.xy *= bumpiness;

	// Calculate the world space normals
	mat3 tbn = mat3(
		normalize(m_tangent),
		normalize(m_bitangent),
		normalize(m_normal));
	vec3 normal = normalize(tbn * normalize(bumps));

	// Create the data structs
	SunData sun = SunData(light_dir, light_color, light_strength);
	SurfaceData surface = SurfaceData(diffuse, mask, normal, m_position);
	vec3 view = normalize(camera - m_position);
	CameraData camera = CameraData(view, normalize(view + light_dir), camera);
	
	// Color of the final result
	frag = brdf(surface, camera, sun);
}