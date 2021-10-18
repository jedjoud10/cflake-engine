// Create some volumetric fog!
// The result of the raymarching
struct VolumetricResult {
    vec3 color;
    float depth;
};
const int STEP_COUNT = 256;
// SDF function
float scene(vec3 point) {
    return length(point) - 4;
}
float map(float x, float ra, float rb, float r2a, float r2b) {
    // https://stackoverflow.com/questions/3451553/value-remapping
    return r2a + (x - ra) * (r2b - r2a) / (rb - ra);
}
VolumetricResult volumetric(vec3 camera_position, vec3 pixel_forward, vec2 nf_planes, mat4 vp_matrix) {
    // Starting point at camera
    vec3 point = camera_position;
    vec3 color = vec3(0, 0, 0);
    float min_dist = 1000;
    for(int i = 0; i < STEP_COUNT; i++) {        
        // Offset the point using the forward vector and step size
        float d = scene(point);
        point += pixel_forward * 0.06;
        min_dist = min(d, min_dist);
        if (d < 0.1) {
            color = vec3(1, 0, 0);
            // Calculate the linear depth
            float z = (vp_matrix * vec4(point, 1)).z;
            return VolumetricResult(point, z);
        }
    }
    return VolumetricResult(color, 0.0);
}