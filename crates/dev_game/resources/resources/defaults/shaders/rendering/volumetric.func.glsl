// Create some volumetric fog!
const int STEP_COUNT = 16;
// SDF function
float scene(vec3 point) {
    return length(point) - 30;
}
vec3 volumetric(vec3 camera_position, vec3 camera_forward, vec2 uv_coords) {
    // Starting point at camera
    vec3 point = camera_position;
    // Final color
    vec3 color = vec3(0, 0, 0);
    float min_dist = 0;
    for(int i = 0; i < STEP_COUNT; i++) {        
        // Offset the point using the forward vector and step size
        point += camera_forward * (1.0/float(STEP_COUNT)) * 2.0;
        float d = scene(point);
        min_dist = min(d, min_dist);
        if (d < 0) {
            color = vec3(1, 0, 0) * -min_dist / 20.0;
            return color;
        }
    }
    return vec3(1, 1, 1) * min_dist / 20.0;
}