// Calculate the color of a fragment using the sky gradient texture
vec3 sky(vec3 normal) {    
    /*
    #define PI 3.1415926538
    float u = atan(normal.x, normal.z) / (2*PI) + 0.5;
    float v = normal.y * 0.5 + 0.5;
    return texture(sky_texture, vec2(u, v)).rgb;
    */
    return texture(sky_gradient, vec2(time_of_day, normal.y)).xyz;
}