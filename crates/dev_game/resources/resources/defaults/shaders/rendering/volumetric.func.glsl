// Create some volumetric fog!
// The result of the raymarching
struct VolumetricResult {
    vec3 color;
    float depth;
};
const int STEP_COUNT = 256;
// SDF function
float scene(vec3 point) {
    float x = length(point) - 4;
    float y = length(point - 10) - 10; 
    return min(x, y);
}
float map(float x, float ra, float rb, float r2a, float r2b) {
    // https://stackoverflow.com/questions/3451553/value-remapping
    return r2a + (x - ra) * (r2b - r2a) / (rb - ra);
}
VolumetricResult volumetric(vec3 camera_position, vec3 pixel_forward, vec3 pixel_forward_projection, vec2 nf_planes) {
    // Starting point at camera
    vec3 point = camera_position;
    vec3 color = vec3(0, 0, 0);
    float min_dist = 1000;
    for(int i = 0; i < STEP_COUNT; i++) {        
        // Offset the point using the forward vector and step size
        float d = scene(point);
        point += pixel_forward * d;
        min_dist = min(d, min_dist);
        if (d < 0.08) {
            color = vec3(1, 0, 0);
            // Cos
            float cos_a = dot(vec3(0, 0, -1), pixel_forward_projection);
            // Distance 
            float d_depth = distance(point, camera_position) * cos_a;
            d_depth = map(d_depth, nf_planes.x, nf_planes.y, 0, 1);
            // Calculate the linear depth
            return VolumetricResult(point, d_depth);
        }
    }
    return VolumetricResult(color, 0.0);
}