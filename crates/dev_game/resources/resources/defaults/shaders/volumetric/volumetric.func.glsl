// Create some volumetric fog!
#include "defaults\shaders\others\hashes.func.glsl"
// The result of the raymarching
struct VolumetricResult {
    vec3 color;
    float depth;
};
const int STEP_COUNT = 64;
const float THRESHOLD = 0.01;
const float NORMAL_OFFSET = 0.3;
// Sampling the SDF texture
float scene(vec3 point, sampler3D sdf_tex) {
    float d = texture(sdf_tex, -point * 0.1).x - 0.2;
    
    d = max(point.y, d);
    d = max(-point.y - 5, d);
    
    return d;
}
float map(float x, float ra, float rb, float r2a, float r2b) {
    // https://stackoverflow.com/questions/3451553/value-remapping
    return r2a + (x - ra) * (r2b - r2a) / (rb - ra);
}
VolumetricResult volumetric(vec3 camera_position, vec2 uvs, vec3 pixel_forward, vec3 pixel_forward_projection, vec2 nf_planes, sampler3D sdf_tex) {
    // Starting point at camera
    vec3 point = camera_position + pixel_forward;
    float last_d = scene(point, sdf_tex);
    for(int i = 0; i < STEP_COUNT; i++) { 
        // Offset the point using the forward vector and a dynamic step size
        point += pixel_forward * last_d;
        float d = scene(point, sdf_tex);
        last_d = d;
        // We hit the surface!!
        if (d < THRESHOLD) {
            vec3 color = vec3(1, 1, 1);
            // Cos
            float cos_a = dot(vec3(0, 0, -1), pixel_forward_projection);
            // Distance 
            float d_depth = distance(point, camera_position) * cos_a;
            d_depth = map(d_depth, nf_planes.x, nf_planes.y, 0, 1);
            
            
            // Calculate the normal at the specific intersection point
            float nd1 = scene(point + vec3(NORMAL_OFFSET*vec3(1, 0, 0)), sdf_tex);
            float nd2 = scene(point + vec3(NORMAL_OFFSET*vec3(0, 1, 0)), sdf_tex);
            float nd3 = scene(point + vec3(NORMAL_OFFSET*vec3(0, 0, 1)), sdf_tex);
            /*

            float nd4 = scene(point - vec3(NORMAL_OFFSET*vec3(1, 0, 0)), sdf_tex);
            float nd5 = scene(point - vec3(NORMAL_OFFSET*vec3(0, 1, 0)), sdf_tex);
            float nd6 = scene(point - vec3(NORMAL_OFFSET*vec3(0, 0, 1)), sdf_tex);
            */
            vec3 normal = normalize(vec3(nd1-d, nd2-d, nd3-d));
            
            // Calculate the linear depth
            return VolumetricResult(normal, d_depth);     
        }
    }
    // No hits
    return VolumetricResult(vec3(0, 0, 0), 0.0);
    
}