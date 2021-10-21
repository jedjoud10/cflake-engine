// Create some volumetric fog!
#include "defaults\shaders\others\hashes.func.glsl"
// The result of the raymarching
struct VolumetricResult {
    vec3 color;
    float depth;
};
const int STEP_COUNT = 512;
const float MAX_DISTANCE = 500;
const float THRESHOLD = 0.03;
const float NORMAL_OFFSET = 0.05;
// Sampling the SDF texture
float scene(vec3 point, sampler3D sdf_tex, float time) {
    vec3 scale = vec3(1, 1, 1) / 30;
    float d = texture(sdf_tex, -point * scale).x * 1.0;
    d = min(point.y, d);
    d = max(point.y - 2, d);
    //d = max(-point.y - 10, d);
    return d;
}
float map(float x, float ra, float rb, float r2a, float r2b) {
    // https://stackoverflow.com/questions/3451553/value-remapping
    return r2a + (x - ra) * (r2b - r2a) / (rb - ra);
}
VolumetricResult volumetric(vec3 camera_position, vec2 uvs, vec3 pixel_forward, vec3 pixel_forward_projection, vec2 nf_planes, sampler3D sdf_tex, float time) {
    // Starting point at camera
    vec3 point = camera_position + pixel_forward;    
    float d = scene(point, sdf_tex, time);
    float last_d = d;
    vec3 last_point = point;
    for(int i = 0; i < STEP_COUNT; i++) { 
        // Max distance
        if (distance(point, camera_position) > MAX_DISTANCE || (point.y > 2 && pixel_forward.y > 0) || (point.y < 0 && pixel_forward.y < 0)) {
            break;
        }
        // Offset the point using the forward vector and a dynamic step size
        d = scene(point, sdf_tex, time);
        point += pixel_forward * d;
        // We hit the surface!!
        if (d < THRESHOLD) {
            // Get the exact intersection point
            vec3 color = vec3(1, 1, 1);
            // Cos
            float cos_a = dot(vec3(0, 0, -1), pixel_forward_projection);
            // Distance 
            float d_depth = distance(point, camera_position) * cos_a;
            d_depth = map(d_depth, nf_planes.x, nf_planes.y, 0, 1);
            
            
            // Calculate the normal at the specific intersection point
            /*
            float nd1 = scene(point + vec3(NORMAL_OFFSET*vec3(1, 0, 0)), sdf_tex, time);
            float nd2 = scene(point + vec3(NORMAL_OFFSET*vec3(0, 1, 0)), sdf_tex, time);
            float nd3 = scene(point + vec3(NORMAL_OFFSET*vec3(0, 0, 1)), sdf_tex, time);

            float nd4 = scene(point - vec3(NORMAL_OFFSET*vec3(1, 0, 0)), sdf_tex);
            float nd5 = scene(point - vec3(NORMAL_OFFSET*vec3(0, 1, 0)), sdf_tex);
            float nd6 = scene(point - vec3(NORMAL_OFFSET*vec3(0, 0, 1)), sdf_tex);
            vec3 normal = normalize(vec3(nd1-d, nd2-d, nd3-d));
            */
            vec3 ic = float(i) / float(STEP_COUNT) * vec3(1, 1, 1);
            // Calculate the linear depth
            return VolumetricResult(ic, d_depth);     
        }

        // Update values for the next iteration
        last_d = d;
        last_point = point;
    }
    // No hits
    return VolumetricResult(vec3(0, 0, 0), 0.0);
    
}