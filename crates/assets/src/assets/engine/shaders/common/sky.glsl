// Calculate a procedural sky color based on a multitude of gradients
vec3 calculate_sky_color(
    vec3 normal,
    vec3 sun
) {
    // Get up component of vector and remap to 0 - 1 range
    float up = normal.y * 0.5 + 0.5;

    // Define color mapping values
    const vec3 dark_blue = pow(vec3(0.137,0.263,0.463), vec3(2.2));
    const vec3 light_blue = pow(vec3(0.533,0.733,0.857), vec3(2.2));
    const vec3 orange = pow(vec3(247.0, 134.0, 64.0) / 255.0, vec3(2.2));

    // Do some color mapping (day color)
    vec3 day_color = mix(light_blue, dark_blue, max(up, 0.2));

    // Do some color mapping (sunset color)
    vec3 sunset_color = mix(orange, dark_blue, max(up, 0.2));

    // Mix in night color
    float time_of_day = min(max(-sun.y, 0), 0.25) * 4;
    day_color = mix(day_color, vec3(0.001), 1-time_of_day);

    return day_color;
}