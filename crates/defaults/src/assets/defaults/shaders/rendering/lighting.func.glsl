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
	// Calculate the diffuse lighting
	float light_val = max(dot(normal, normalize(sunlight_dir)), 0) * sunlight_strength; 

	// Used for ambient lighting
	float ambient_lighting_strength = 0.1;
	float sky_light_val = dot(normal, vec3(0, 1, 0)); 
	vec3 ambient_lighting_color = calculate_sky_color(sky_texture, sky_light_val, time_of_day);

	// Add everything
	vec3 ambient_lighting = (ambient_lighting_color + diffuse * 2.0) * ambient_lighting_strength;
	vec3 color = ambient_lighting;
	color += (1 - in_shadow) * (diffuse * light_val);
	
	// Calculate some specular
	float specular_val = pow(clamp(dot(pixel_dir, reflect(sunlight_dir, normal)), 0, 1), 256);
	//color += specular_val * 0.6 * (1 - in_shadow);

    return color + emissive;
}