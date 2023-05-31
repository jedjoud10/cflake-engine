#version 460 core
layout(location = 0) out vec4 frag;


// Fetch the G-Buffer data
layout(set = 1, binding = 0) uniform texture2D gbuffer_position_map;
layout(set = 1, binding = 1) uniform texture2D gbuffer_albedo_map;
layout(set = 1, binding = 2) uniform texture2D gbuffer_normal_map;
layout(set = 1, binding = 3) uniform texture2D gbuffer_mask_map;

// UBO that contains the current scene information
#include <engine/shaders/common/conversions.glsl>
#include <engine/shaders/common/camera.glsl>
layout(set = 0, binding = 2) uniform SceneUniform {
    // Sun related parameters
    vec4 sun_direction;
    vec4 sun_color;

    // Ambient color of the environment
    float ambient_color_strength;
    
    // Procedural sun circle parameters
    float sun_circle_strength;
    float sun_circle_size;
    float sun_circle_fade;
} scene;

// Post processing settings
layout(set = 0, binding = 3) uniform PostProcessUniform {
    float exposure;
	float gamma;
	float vignette_strength;
	float vignette_size;
	uint tonemapping_mode;
	float tonemapping_strength;
    uint gbuffer_debug;
} post_processing;

/*
// Environment texture map
layout(set = 0, binding = 2) uniform textureCube environment_map;
layout(set = 0, binding = 3) uniform sampler environment_map_sampler;
*/

/*
// UBO set globally at the start of the frame
layout(set = 0, binding = 4) uniform ShadowUniform {
    float strength;
    float spread;
} shadow_parameters;

// Contains all the lightspace matrices for each cascade
layout(set = 0, binding = 5) uniform ShadowLightSpaceMatrices {
    mat4 matrices[4];
} shadow_lightspace_matrices;

// Contains all the cascade plane distances
layout(set = 0, binding = 6) uniform ShadowPlaneDistances {
    vec4 distances;
} cascade_plane_distances;

// Shadow-map texture map
layout(set = 0, binding = 7) uniform texture2DArray shadow_map;


// Sample a single shadow texel at the specified pixel coords
float sample_shadow_texel(
    uint layer,
    ivec2 pixel,
    float compare
) {
    float closest = texelFetch(shadow_map, ivec3(pixel, int(layer)), 0).r;
    return (compare > closest) ? 1.0 : 0.0;
}

// Calculate a linearly interpolated shadow value
float shadow_linear(
    uint layer,
    vec2 uvs,
    uint size,
    float compare
) {
    // Get a quad that contains the binary values
    ivec2 pixel = ivec2(uvs.xy * size);
    float uv0 = sample_shadow_texel(layer, pixel, compare);
    float uv1 = sample_shadow_texel(layer, pixel + ivec2(1, 0), compare);
    float uv2 = sample_shadow_texel(layer, pixel + ivec2(0, 1), compare);
    float uv3 = sample_shadow_texel(layer, pixel + ivec2(1, 1), compare);

    // Interpolate results in the x axis
    vec2 frac = fract(uvs * vec2(size));
    float bottom = mix(uv0, uv1, frac.x);
    float top = mix(uv2, uv3, frac.x);

    // Interpolate results in the y axis
    return mix(bottom, top, frac.y);
}

// Check if a pixel is obscured by the shadow map
float calculate_shadowed(
    vec3 position,
    float depth,
    vec3 normal,
    vec3 light_dir,
    vec3 camera
) {
    return 0.0;

    // Taken from a comment by Octavius Ace from the same learn OpenGL website 
    vec4 res = step(cascade_plane_distances.distances, vec4(depth));
    uint layer = uint(res.x + res.y + res.z + res.w);
    
    // Get the proper lightspace matrix that we will use
    mat4 lightspace = shadow_lightspace_matrices.matrices[layer];
    
    // Transform the world coordinates to NDC coordinates 
    float perpendicularity = pow(1 - abs(dot(normal, light_dir)), 2) * 2;
    vec4 ndc = lightspace * vec4(position + normal * perpendicularity * 0.08, 1.0); 
    float factor = pow(1.35, layer*4);
    float bias = -0.00002;
    bias *= factor;

    // Project the world point into uv coordinates to read from
    vec3 uvs = ndc.xyz / ndc.w;
    uvs.xy *= 0.5;
    uvs.xy += 0.5;
    uvs.y = 1-uvs.y;
    float current = uvs.z;

    // Get texture size
    uint size = uint(textureSize(shadow_map, 0).x);
    return shadow_linear(layer, uvs.xy, size, current + bias);
}

*/

// UBO that contains the current monitor/window information
layout(set = 0, binding = 8) uniform WindowUniform {
    // Dimensions of the window
    uint width;
    uint height;
} window;

/*
// Depth map automatically generated when rasterizing the scene
layout(set = 1, binding = 3) uniform texture2D depth_map;
*/


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
	//vec3 ambient = texture(samplerCube(environment_map, environment_map_sampler), surface.normal).rgb;
	//vec3 ambient = calculate_sky_color(surface.normal, -light.backward);
	vec3 ambient = vec3(0);

	// Calculate if the pixel is shadowed
	float depth = abs((camera.view_matrix * vec4(surface.position, 1)).z);
	float shadowed = 0;
    //float shadowed = calculate_shadowed(surface.position, depth, surface.surface_normal, light.backward, camera.position);	
	//return vec3(shadowed);

	// Calculate diffuse and specular
	vec3 brdf = kd * (surface.diffuse / PI) + specular(surface.f0, surface.roughness, camera.view, light.backward, surface.normal, camera.half_view) * (1-shadowed);
	vec3 lighting = vec3(max(dot(light.backward, surface.normal), 0.0)) * (1-shadowed);
	lighting += 0.1 * surface.visibility + ambient * 0.05;

	// TODO: IBL
	brdf = brdf * light.color * lighting;
	brdf += fresnelRoughness(surface.f0, camera.view, surface.normal, surface.roughness) * 0.40;
	return brdf;
}

void main() {
    // Fetch G-Buffer values
	vec3 position = texelFetch(gbuffer_position_map, ivec2(gl_FragCoord.xy), 0).rgb;
	vec3 albedo = texelFetch(gbuffer_albedo_map, ivec2(gl_FragCoord.xy), 0).rgb;
    vec3 normal = texelFetch(gbuffer_normal_map, ivec2(gl_FragCoord.xy), 0).rgb;
    vec3 mask = texelFetch(gbuffer_mask_map, ivec2(gl_FragCoord.xy), 0).rgb;

    float roughness = clamp(mask.g, 0.02, 1.0);
	float metallic = clamp(mask.b, 0.01, 1.0);
	float visibility = clamp(mask.r, 0.0, 1.0);
	vec3 f0 = mix(vec3(0.04), albedo, metallic);

	// Create the data structs
	SunData sun = SunData(-scene.sun_direction.xyz, scene.sun_color.rgb);
	SurfaceData surface = SurfaceData(albedo, normalize(normal), normal, position, roughness, metallic, visibility, f0);
	vec3 view = normalize(camera.position.xyz - position);
	CameraData camera = CameraData(view, normalize(view - scene.sun_direction.xyz), camera.position.xyz, camera.view, camera.projection);

	// Check if the fragment is shadowed
	vec3 color = brdf(surface, camera, sun);
    
    // Increase exposure
	color *= post_processing.exposure;
	color = max(color, vec3(0));
	vec3 tonemapped = color;

	/*
	Reinhard,
	ReinhardJodie,
	ACES,
	Clamp,
	*/

	// Handle tonemapping mode
	switch(post_processing.tonemapping_mode) {
		case 0:
			tonemapped = reinhard(color);
			break;
		case 1:
			tonemapped = reinhard_jodie(color);
			break;
		case 2:
			tonemapped = aces(color);
			break;
		case 3:
			tonemapped = min(color, vec3(1));
			break;
	}

    // Optional G-Buffer debug
    switch(post_processing.gbuffer_debug) {
		case 0:
            color = position;
			tonemapped = position;
			break;
		case 1:
			color = albedo;
			tonemapped = albedo;
            break;
		case 2:
			color = max(normal, vec3(0));
			tonemapped = max(normal, vec3(0));
            break;
		case 3:
			color = mask;
			tonemapped = mask;
            break;
	}
	
	// Apply gamma correction
	tonemapped = mix(color, tonemapped, post_processing.tonemapping_strength);
	color = pow(tonemapped, vec3(1.0 / post_processing.gamma));
	frag = vec4(color, 0);
}