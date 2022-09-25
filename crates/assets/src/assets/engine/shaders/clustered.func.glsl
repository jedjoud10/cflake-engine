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