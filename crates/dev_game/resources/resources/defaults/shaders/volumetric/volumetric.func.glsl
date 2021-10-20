// Create some volumetric fog!
// The result of the raymarching
struct VolumetricResult {
    vec3 color;
    float depth;
};
const int STEP_COUNT = 64;
const float THRESHOLD = 0.1;
// SDF function
float scene(vec3 point) {
    return length(point) - 5;
}
float map(float x, float ra, float rb, float r2a, float r2b) {
    // https://stackoverflow.com/questions/3451553/value-remapping
    return r2a + (x - ra) * (r2b - r2a) / (rb - ra);
}
VolumetricResult volumetric(vec3 camera_position, vec3 pixel_forward, vec3 pixel_forward_projection, vec2 nf_planes) {
    // Starting point at camera
    vec3 point = camera_position;
    float last_d = scene(camera_position + pixel_forward);
    for(int i = 0; i < STEP_COUNT; i++) { 
        // Offset the point using the forward vector and a dynamic step size
        point += pixel_forward * last_d;
        float d = scene(point);
        last_d = d;
        // We hit the surface!!
        if (d < THRESHOLD) {
            vec3 color = vec3(1, 1, 1);
            // Cos
            float cos_a = dot(vec3(0, 0, -1), pixel_forward_projection);
            // Distance 
            float d_depth = distance(point, camera_position) * cos_a;
            d_depth = map(d_depth, nf_planes.x, nf_planes.y, 0, 1);
            // Calculate the linear depth
            return VolumetricResult(color, d_depth);     
        }
    }
    // No hits
    return VolumetricResult(vec3(0, 0, 0), 0.0);
    
}