// Calculate the color of a fragment using the sky gradient texture
vec3 calculate_sky_color(sampler2D sky_texture, float sun_up_factor, float offset) {
    return texture(sky_texture, vec2(offset, sun_up_factor)).xyz;
}