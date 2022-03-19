// Compute lighting for a single pixel on the screen
vec3 compute_lighting(
    vec3 sunlight_dir,
    float sunlight_strength,
    vec3 diffuse,
    vec3 normal,
    vec3 emissive,
    vec3 position,
    vec3 pixel_dir,
    float in_shadow,
    sampler2D sky_texture,
    float time_of_day) {       	
	// Pixel direction reflected by the surface
	vec3 reflected = reflect(pixel_dir, normal);
	float fresnel = max(dot(reflected, pixel_dir), 0);

	// Calculate the diffuse lighting
	float light_val = max(dot(normal, normalize(sunlight_dir)), 0) * sunlight_strength * 1.3; 

	// Used for ambient lighting
	float ambient_lighting_strength = 0.02 * (normal.y * 0.5 + 0.5);
	float sky_light_val = dot(normal, vec3(0, 1, 0)); 
	vec3 ambient_lighting_color = calculate_sky_color(sky_texture, normal, sky_light_val, time_of_day);

	// Add everything
	vec3 ambient_lighting = diffuse * 0.07 + ambient_lighting_color * ambient_lighting_strength;
	vec3 color = ambient_lighting + pow(fresnel, 3.0) * 0.02;
	color += (1 - in_shadow) * (diffuse * light_val);
	
	// Calculate some specular
	/*
	float specular_val = pow(clamp(dot(pixel_dir, reflect(sunlight_dir, normal)), 0, 1), 512);
	color += specular_val * 1.0 * (1 - in_shadow);
	*/
    return color + emissive;
}