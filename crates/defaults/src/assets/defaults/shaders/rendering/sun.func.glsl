// Calculate the sun's strength using the sun's dot product
float calculate_sun_strength(float sun_up_factor) {
    return clamp(sun_up_factor * 6 - 2.4, 0, 1);
}