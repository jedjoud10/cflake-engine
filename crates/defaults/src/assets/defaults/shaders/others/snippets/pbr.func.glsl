// PBR code automatically implements main camera code
#load camera

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
vec3 brdf(vec3 n, vec3 v, vec3 l, vec3 h, float roughness, float metallic, vec3 diffuse, vec3 emissive, SunData sun) {
	// Constants
	roughness = max(roughness, 0.05);
	metallic = pow(metallic, 5);
	vec3 f0 = mix(vec3(0.04), diffuse, metallic);
	
	// Ks and Kd
	vec3 ks = fresnel(f0, v, h, n);
	vec3 kd = (1 - ks) * (1 - metallic);

	// Le diffuse and specular
	vec3 brdf = kd * (diffuse / PI) + specular(f0, roughness, v, l, n, h);
	vec3 outgoing = emissive + brdf * sun.color * sun.strength * max(dot(l, n), 0.0);

	return outgoing;
}

// Calculate the shaded color for a single pixel 
vec3 shade_pbr(SunData sun, PixelData pixel) {   
    // Create a camera from the uniforms
    CameraData camera = CameraData(_cam_pos, _cam_dir, _pv_matrix);

	// Main vectors
	vec3 n = normalize(pixel.normal);
	vec3 v = normalize(camera.position - pixel.position);
	vec3 l = -normalize(sun.direction);
	vec3 h = normalize(v + l);

	// The shaded pixel color
	vec3 color = brdf(n, v, l, h, pixel.roughness, pixel.metallic, pixel.diffuse, pixel.emissive, sun) * (1 - pixel.in_shadow);

	// Ambient color
	color += 0.03 * pixel.diffuse * pixel.ao;
	return color;
}