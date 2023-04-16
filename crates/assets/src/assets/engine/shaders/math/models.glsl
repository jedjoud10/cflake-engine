#define PI 3.1415926538

// Literally the whole implementation is stolen from
// https://www.youtube.com/watch?v=RRE-F57fbXw&ab_channel=VictorGordan
// and https://learnopengl.com/PBR/Lighting

// Normal distribution function
// GGX/Trowbridge-reitz model
float ndf(float roughness, vec3 n, vec3 h) {
	float a = roughness * roughness;
	float a2 = a * a;

	float n_dot_h = max(dot(n, h), 0.0);	
	float n_dot_h2 = n_dot_h * n_dot_h;	

	float semi_denom = n_dot_h2 * (a2 - 1.0) + 1.0;
	float denom = PI * semi_denom * semi_denom;
	return a2 / denom;
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
vec3 fresnel(vec3 f0, vec3 h, vec3 v) {
	float cosTheta = max(dot(h, v), 0.0);
    return f0 + (1.0 - f0) * pow (1.0 - cosTheta, 5.0);
}
/*
vec3 fresnel(vec3 f0, vec3 v, vec3 h) {
	float cosTheta = clamp(1.0 - max(dot(v, h), 0), 0, 1);
	return f0 + (1.0 - f0) * pow(cosTheta, 5.0);
}
*/

// Fresnel function with roughness
vec3 fresnelRoughness(vec3 f0, vec3 v, vec3 x, float roughness) {
	float cosTheta = clamp(1.0 - max(dot(v, x), 0), 0, 1);
	return f0 + (max(vec3(1.0 - roughness), f0) - f0) * pow(cosTheta, 5.0);
}

// Cook-torrence model for specular
vec3 specular(vec3 f0, float roughness, vec3 v, vec3 l, vec3 n, vec3 h) {
	vec3 num = ndf(roughness, n, h) * gsf(roughness, n, v, l) * fresnel(f0, v, h);
	float denom = 4 * max(dot(v, n), 0.0) * max(dot(l, n), 0.0) + 0.01;
	return num / denom;
}

// Sun data struct
struct SunData {
	vec3 backward;
	vec3 color;
};

// Camera data struct
struct CameraData {
	vec3 view;
	vec3 half_view;
	vec3 position;
	mat4 view_matrix;
	mat4 proj_matrix;
};

// Surface data struct 
struct SurfaceData {
	vec3 diffuse;
	vec3 normal;
	vec3 surface_normal;
	vec3 position;
	float roughness;
	float metallic;
	float visibility;
	vec3 f0;
};

// Bidirectional reflectance distribution function, aka PBRRRR
vec3 brdf(
	SurfaceData surface,
	CameraData camera,
	SunData light
) {
	// Calculate kS and kD
	// TODO: Fix this shit it's fucked
	//vec3 ks = fresnel(surface.f0, camera.half_view, camera.view);
	vec3 ks = vec3(0);
	vec3 kd = (1 - ks) * (1 - surface.metallic);

	// Calculate ambient sky color
	vec3 ambient = calculate_sky_color(-surface.normal, -light.backward);

	// Calculate if the pixel is shadowed
	float depth = abs((camera.view_matrix * vec4(surface.position, 1)).z);
	float shadowed = calculate_shadowed(surface.position, depth, surface.surface_normal, light.backward, camera.position);	

	// Calculate diffuse and specular
	vec3 brdf = kd * (surface.diffuse / PI) + specular(surface.f0, surface.roughness, camera.view, light.backward, surface.normal, camera.half_view) * (1-shadowed);
	vec3 lighting = vec3(max(dot(light.backward, surface.normal), 0.0)) * (1-shadowed);
	lighting += ambient * 0.3 * surface.visibility;
	brdf = brdf * light.color * lighting;
	brdf += calculate_sky_color(reflect(camera.view, -surface.normal), light.backward) * fresnelRoughness(surface.f0, camera.view, surface.normal, surface.roughness) * 0.20;
	return brdf;
}