

#define PI 3.1415926538

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
vec3 fresnel(vec3 f0, vec3 v, vec3 x) {
	float cosTheta = clamp(1.0 - max(dot(v, x), 0), 0, 1);
	return f0 + (1.0 - f0) * pow(cosTheta, 5.0);
}

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