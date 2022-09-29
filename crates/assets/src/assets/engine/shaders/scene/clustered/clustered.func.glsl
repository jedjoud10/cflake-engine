// A shading cluster will contain an offset and length that will be used to reference a contiguous set of cluster light indices
struct ShadingCluster {
    uint offset;
    uint len;
};

// This is a single point light that will be sent from the CPU
struct PackedPointLight {
    vec4 color;
    vec4 position_attenuation;
};


// Calculate the attenuation of a light using it's distance from the fragment
float calculate_attenuation(float dist) {
    return 1.0 / (1.0 + dist*dist);
}