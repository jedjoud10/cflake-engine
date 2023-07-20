#version 460 core
layout(location = 0) out vec4 frag;

// Fetch the G-Buffer data
layout(set = 1, binding = 1) uniform texture2D gbuffer_albedo_map;
layout(set = 1, binding = 2) uniform texture2D gbuffer_normal_map;
layout(set = 1, binding = 3) uniform texture2D gbuffer_mask_map;
layout(set = 1, binding = 4) uniform texture2D depth_map;

// UBO that contains the current scene information
#include <engine/shaders/common/conversions.glsl>
#include <engine/shaders/common/camera.glsl>
#include <engine/shaders/noises/common.glsl>

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
	float vignette_strength;
	float vignette_size;
	uint tonemapping_mode;
	float tonemapping_strength;
    uint gbuffer_debug;
	float cc_wb_temperature;
	vec4 cc_gain;
	vec4 cc_lift;
	vec4 cc_gamma;
} post_processing;

// Environment texture map
layout(set = 0, binding = 4) uniform textureCube environment_map;
layout(set = 0, binding = 5) uniform sampler environment_map_sampler;

// Diffuse IBL map
layout(set = 0, binding = 6) uniform textureCube ibl_diffuse_map;
layout(set = 0, binding = 7) uniform sampler ibl_diffuse_map_sampler;

// UBO set globally at the start of the frame
layout(set = 0, binding = 8) uniform ShadowUniform {
    float strength;
    float spread;
	float base_bias;
	float bias_bias;

	float bias_factor_base;
	float max_distance;
	float _padding1;
	float _padding2;

	vec4 distances;
} shadow_parameters;

// Contains all the lightspace matrices for each cascade
layout(set = 0, binding = 9) uniform ShadowLightSpaceMatrices {
    mat4 matrices[4];
} shadow_lightspace_matrices;

// Shadow-map texture map and its sampler
layout(set = 0, binding = 10) uniform texture2DArray shadow_map;
layout(set = 0, binding = 11) uniform sampler shadow_map_sampler;

// UBO that contains the current monitor/window information
layout(set = 0, binding = 12) uniform WindowUniform {
    // Dimensions of the window
    uint width;
    uint height;
} window;

// Calculate a linearly interpolated shadow value
float shadow_linear(
    uint layer,
    vec2 uvs,
    uint size,
    float compare
) {
	return texture(sampler2DArrayShadow(shadow_map, shadow_map_sampler), vec4(uvs, layer, compare)).r;
}

vec2 vogel_disk_sample(int sampleIndex, int samplesCount, float phi)
{
  float GoldenAngle = 2.4f;

  float r = sqrt(sampleIndex + 0.5f) / sqrt(samplesCount);
  float theta = sampleIndex * GoldenAngle + phi;
  
  return vec2(r * cos(theta), r * sin(theta));
}

// Check if a pixel is obscured by the shadow map
float calculate_shadowed(
    vec3 position,
    float depth,
    vec3 normal,
    vec3 light_dir,
    vec3 camera
) {
	// We're a lot tighter with the shadow frustum now so we have to fallback to this
	// This definitely could be simplified idk how
	uint layer = int(-1);
	for (int i = 0; i <= 4; ++i)
	{
		mat4 lightspace = shadow_lightspace_matrices.matrices[i];
    	vec4 ndc = lightspace * vec4(position, 1.0); 
	    if (abs(ndc.x) < 1.0 && abs(ndc.y) < 1.0)
	    {
	        layer = i;
	        break;
	    }
	}

	if (layer == int(-1)) {
		return 0.0;
	}
	
	/*
	// Taken from a comment by Octavius Ace from the same learn OpenGL website 
    vec4 res = step(shadow_parameters.distances, vec4(distance(position, camera)) / 2.0);
    uint layer = uint(res.x + res.y + res.z + res.w);
	*/

    // Transform the world coordinates to NDC coordinates 
	mat4 lightspace = shadow_lightspace_matrices.matrices[layer];
    vec4 ndc = lightspace * vec4(position, 1.0); 
    float factor = pow(shadow_parameters.bias_factor_base, layer*4);
    float bias = shadow_parameters.base_bias;
    bias *= factor;
	bias -= shadow_parameters.bias_bias;

    // Project the world point into uv coordinates to read from
    vec3 uvs = ndc.xyz / ndc.w;
    uvs.xy *= 0.5;
    uvs.xy += 0.5;
    uvs.y = 1-uvs.y;
    float current = uvs.z;

    // Get texture size
    uint size = uint(textureSize(shadow_map, 0).x);
	float shadowed = 0.0;

	// Stratified poisson disk from http://www.opengl-tutorial.org/intermediate-tutorials/tutorial-16-shadow-mapping/
	const vec2 poisson_disk[16] = vec2[]( 
	   vec2( -0.94201624, -0.39906216 ), 
	   vec2( 0.94558609, -0.76890725 ), 
	   vec2( -0.094184101, -0.92938870 ), 
	   vec2( 0.34495938, 0.29387760 ), 
	   vec2( -0.91588581, 0.45771432 ), 
	   vec2( -0.81544232, -0.87912464 ), 
	   vec2( -0.38277543, 0.27676845 ), 
	   vec2( 0.97484398, 0.75648379 ), 
	   vec2( 0.44323325, -0.97511554 ), 
	   vec2( 0.53742981, -0.47373420 ), 
	   vec2( -0.26496911, -0.41893023 ), 
	   vec2( 0.79197514, 0.19090188 ), 
	   vec2( -0.24188840, 0.99706507 ), 
	   vec2( -0.81409955, 0.91437590 ), 
	   vec2( 0.19984126, 0.78641367 ), 
	   vec2( 0.14383161, -0.14100790 ) 
	);

	for (int i = 0; i < 16; i++) {
		//vec2 offset = poisson_disk[i] * shadow_parameters.spread;
		vec2 offset = vogel_disk_sample(i, 16, random(gl_FragCoord.xy)) * shadow_parameters.spread;
		float weight = 1.0 / length(offset);
		shadowed += shadow_linear(layer, uvs.xy + offset * 0.004 /* * weight */, size, current + bias);
	}

    return (shadowed / 16) * shadow_parameters.strength;
}

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
	vec3 num = ndf(roughness, n, h) * gsf(roughness, n, v, l) * fresnel(f0, h, v);
	float denom = 4 * max(dot(v, n), 0.0) * max(dot(l, n), 0.0) + 0.0001;
	return num;
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
	vec3 ks = fresnelRoughness(surface.f0, camera.half_view, camera.view, surface.roughness);
	vec3 kd = (1 - ks) * (1 - surface.metallic);
	
	// Calculate if the pixel is shadowed
	float depth = abs((camera.view_matrix * vec4(surface.position, 1)).z);
    float shadowed = calculate_shadowed(surface.position, depth, surface.surface_normal, light.backward, camera.position);	
	//float shadowed = 0.0;

	vec3 specular = specular(surface.f0, surface.roughness, camera.view, light.backward, surface.normal, camera.half_view);
	vec3 brdf = kd * (surface.diffuse) + specular + fresnel(surface.f0, camera.half_view, camera.view);
	vec3 lighting = vec3(max(dot(light.backward, surface.normal), 0.0)) * (1 - shadowed);
	brdf *= lighting * light.color;

	// Diffuse Irradiance IBL
	vec3 irradiance = texture(samplerCube(ibl_diffuse_map, ibl_diffuse_map_sampler), surface.normal).xyz;
	vec3 ambient = irradiance * surface.diffuse * kd * surface.visibility;
	// + vec3(clamp(dot(reflect(camera.view, surface.normal), camera.view), 0, 1)) * 0.04
	return brdf + ambient * 1.5;
}

// https://gamedev.stackexchange.com/questions/92015/optimized-linear-to-srgb-glsl
// Converts a color from linear light gamma to sRGB gamma
vec3 from_linear(vec3 linearRGB)
{
    bvec3 cutoff = lessThan(linearRGB, vec3(0.0031308));
    vec3 higher = vec3(1.055)*pow(linearRGB, vec3(1.0/2.4)) - vec3(0.055);
    vec3 lower = linearRGB * vec3(12.92);
    return mix(higher, lower, cutoff);
}

// https://www.shadertoy.com/view/4sc3D7
// Valid from 1000 to 40000 K (and additionally 0 for pure full white)
const float PI2 = 6.2831853071;
vec3 color_temperature_to_RGB(const in float temperature) {
  // Values from: http://blenderartists.org/forum/showthread.php?270332-OSL-Goodness&p=2268693&viewfull=1#post2268693   
  mat3 m = (temperature <= 6500.0) ? mat3(vec3(0.0, -2902.1955373783176, -8257.7997278925690),
	                                      vec3(0.0, 1669.5803561666639, 2575.2827530017594),
	                                      vec3(1.0, 1.3302673723350029, 1.8993753891711275)) : 
	 								 mat3(vec3(1745.0425298314172, 1216.6168361476490, -8257.7997278925690),
   	                                      vec3(-2666.3474220535695, -2173.1012343082230, 2575.2827530017594),
	                                      vec3(0.55995389139931482, 0.70381203140554553, 1.8993753891711275)); 
  return mix(clamp(vec3(m[0] / (vec3(clamp(temperature, 1000.0, 40000.0)) + m[1]) + m[2]), vec3(0.0), vec3(1.0)), vec3(1.0), smoothstep(1000.0, 0.0, temperature));
}

void main() {
    // Fetch G-Buffer values
	vec4 albedo_alpha = texelFetch(gbuffer_albedo_map, ivec2(gl_FragCoord.xy), 0);
	vec3 albedo = albedo_alpha.rgb;
	float alpha = albedo_alpha.a;
    vec3 normal = texelFetch(gbuffer_normal_map, ivec2(gl_FragCoord.xy), 0).rgb;
    vec3 mask = texelFetch(gbuffer_mask_map, ivec2(gl_FragCoord.xy), 0).rgb;

	// Get the scaled down coordinates
	float x = gl_FragCoord.x / float(window.width);
	float y = gl_FragCoord.y / float(window.height);

	// Fetch depth ngl
	float non_linear_depth = texelFetch(depth_map, ivec2(gl_FragCoord.xy), 0).r;
	float linear_depth = linearize_depth(non_linear_depth, camera.near_far_vfov_.x, camera.near_far_vfov_.y);

	// Fetch position from depth
	vec4 clip_space_location = vec4(x * 2 - 1, - (y * 2 - 1), non_linear_depth, 1.0);
	vec4 world_pos = inverse(camera.projection * camera.view) * clip_space_location;
	vec3 position = world_pos.xyz / world_pos.w;
	vec3 surface_normal = normalize(cross(dFdy(position), dFdx(position)));

    // Optional G-Buffer debug
    switch(post_processing.gbuffer_debug) {
		case 0:
            frag = vec4(position, 0);
			return;
		case 1:
			frag = vec4(pow(albedo, vec3(1.0 / 2.2)), 0);
            return;
		case 2:
			frag = vec4(max(normal, vec3(0)), 0);
            return;
		case 3:
			frag = vec4(max(surface_normal, vec3(0)), 0);
            return;
		case 4:
			frag = vec4(mask.r);
	        return;
		case 5:
			frag = vec4(mask.g);
	        return;
		case 6:
			frag = vec4(mask.b);
	        return;
		case 7:
			frag = vec4(texture(samplerCube(ibl_diffuse_map, ibl_diffuse_map_sampler), normal).xyz, 0);
	        return;
	}



	// Fetch the color data
	vec2 coords = vec2(x, y);

	vec3 color = vec3(0);

	if (alpha == 1.0) {
		float roughness = clamp(mask.g, 0.02, 1.0);
		float metallic = clamp(mask.b, 0.01, 1.0);
		float visibility = clamp(mask.r, 0.0, 1.0);
		vec3 f0 = mix(vec3(0.04), albedo, metallic);

		// Create the data structs
		SunData sun = SunData(-scene.sun_direction.xyz, scene.sun_color.rgb);
		SurfaceData surface = SurfaceData(albedo, normalize(normal), surface_normal, position, roughness, metallic, visibility, f0);
		vec3 view = normalize(camera.position.xyz - position);
		CameraData camera = CameraData(view, normalize(view - scene.sun_direction.xyz), camera.position.xyz, camera.view, camera.projection);

		// Check if the fragment is shadowed
		color = brdf(surface, camera, sun);
	} else { 
		vec3 dir = -normalize(camera.position.xyz - position);

		// Sample the skybox texture
		color = texture(samplerCube(environment_map, environment_map_sampler), dir).xyz;

		// Create a procedural sun with the scene params
		float sun = dot(dir, -scene.sun_direction.xyz);
		float out_sun = pow(max(sun * 0.3, 0), 3) * 1;
		out_sun += clamp(sun - 0.99912, 0, 1.0) * 1200;
		color += vec3(out_sun);
	}

	// Add fog
	//color = mix(color, vec3(1), clamp(linear_depth / 1000.0, 0, 1));
    
    // Increase exposure
	color *= post_processing.exposure;
	color = max(color, vec3(0));

	// Color grading
	//color = pow(max(vec3(0.0), color * (1.0 + post_processing.cc_gain.rgb - post_processing.cc_lift.rgb) + post_processing.cc_lift.rgb), max(vec3(0.0), 1.0 - post_processing.cc_gamma.rgb));

	// Color temperature mapping from https://www.shadertoy.com/view/4sc3D7
	//color = color * color_temperature_to_RGB(post_processing.cc_wb_temperature); 
	//color *= mix(1.0, dot(color, vec3(0.2126, 0.7152, 0.0722)) / max(dot(color, vec3(0.2126, 0.7152, 0.0722)), 1e-5), 1.0);  

	/*
	Reinhard,
	ReinhardJodie,
	ACES,
	ALU,
	Clamp,
	*/

	// Handle tonemapping mode
	vec3 tonemapped = color;
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
			tonemapped = alu(color);
			break;
		case 4:
			tonemapped = min(color, vec3(1));
			break;
	}
	
	// Apply tonemapping and gamma correction
	tonemapped = mix(color, tonemapped, post_processing.tonemapping_strength);
	color = from_linear(tonemapped);

	// Create a simple vignette
	vec2 uv = vec2(x, y);
	float vignette = length(abs(uv - 0.5));
	vignette += post_processing.vignette_size;
	vignette = clamp(vignette, 0, 1);
	vignette = pow(vignette, 4.0) * clamp(post_processing.vignette_strength, 0.0, 2.0);
	color = mix(color, vec3(0), vignette);
	frag = vec4(color, 0);
}