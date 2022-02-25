// Calculate the color of a fragment using the sky gradient texture
#define PI 3.1415926538
vec3 calculate_sky_color(sampler2D sky_texture, vec3 normal, float sun_up_factor, float time_of_day) {    
    float u = atan(normal.x, normal.z) / (2*PI) + 0.5;
    float v = normal.y * 0.5 + 0.5;
    return vec3(u, v, 0.0);
    //return texture(sky_texture, vec2(time_of_day, sun_up_factor)).xyz;
}