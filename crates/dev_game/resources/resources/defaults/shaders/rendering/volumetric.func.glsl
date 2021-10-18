// Create some volumetric fog!
const int STEP_COUNT = 64;
// SDF function
float scene(vec3 point) {
    return length(point) - 4 + sin(point.x * 2) * 2.0;
}
vec3 volumetric(vec3 camera_position, vec3 pixel_forward) {
    // Starting point at camera
    vec3 point = camera_position;
    vec3 color = vec3(0, 0, 0);
    float min_dist = 1000;
    return pixel_forward;
    for(int i = 0; i < STEP_COUNT; i++) {        
        // Offset the point using the forward vector and step size
        point += pixel_forward * 0.8;
        float d = scene(point);
        min_dist = min(d, min_dist);
        if (d < 0) {
            color = vec3(1, 0, 0);
            return color;
        }
    }
    return vec3(1, 1, 1) * min_dist / 20.0;
}