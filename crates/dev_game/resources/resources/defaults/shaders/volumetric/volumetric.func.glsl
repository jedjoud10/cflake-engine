// Create some volumetric fog!
// The result of the raymarching
struct VolumetricResult {
    vec3 color;
    float depth;
};
const int STEP_COUNT = 256;
const float STEP_SIZE = 0.1;
const float THRESHOLD = 0.0;
// SDF function
float scene(vec3 point) {
    return sdBox(point, vec3(5, 10, 10));
}
float map(float x, float ra, float rb, float r2a, float r2b) {
    // https://stackoverflow.com/questions/3451553/value-remapping
    return r2a + (x - ra) * (r2b - r2a) / (rb - ra);
}
float inverse_lerp(float a, float b, float x) {
    return (x - a) / (b - a);
}
// Calculate the density from the summed distance
float density(float d) {
    return pow(d / 10, 6);
}
VolumetricResult volumetric(vec3 camera_position, vec3 pixel_forward, vec3 pixel_forward_projection, vec2 nf_planes) {
    // Starting point at camera
    vec3 point = camera_position;
    vec3 last_point = point;
    float last_d = 0.0;
    vec3 intersection_point = point;
    // The summed distance of the body that we are passing through
    float summed_distance = 0.0; 
    bool hit = false;
    int hit_counter = 0;
    for(int i = 0; i < STEP_COUNT; i++) { 
        // Quit early
        if (STEP_COUNT * STEP_SIZE > nf_planes.y) {
            break;
        }      
        // Offset the point using the forward vector and constant step size
        point += pixel_forward * STEP_SIZE;
        float d = scene(point);
        if (d < THRESHOLD) {
            // First time
            if (!hit) {
                // Only set the intersection point the first time
                // Linearlly interpolate to find the middle point
                float v = inverse_lerp(last_d, d, THRESHOLD);
                summed_distance += STEP_SIZE * (1-v);
                vec3 new_intersection_point = mix(last_point, point, v);
                intersection_point = new_intersection_point;
            } else {
                summed_distance += STEP_SIZE;
            }
            hit = true;  
            hit_counter++;          
        }
        // Used to interpolate and find the exact surface where the signed distance = 0
        last_d = d;
        last_point = point;
    }
    // Return the correct data 
    if (hit) {
        vec3 color = vec3(1, 1, 1);
        // Cos
        float cos_a = dot(vec3(0, 0, -1), pixel_forward_projection);
        // Distance 
        float d_depth = distance(intersection_point, camera_position) * cos_a;
        d_depth = map(d_depth, nf_planes.x, nf_planes.y, 0, 1);
        // Calculate the linear depth
        return VolumetricResult(color * density(max(summed_distance, 0)), d_depth);
    } else {
        return VolumetricResult(vec3(0, 0, 0), 0.0);
    }
}