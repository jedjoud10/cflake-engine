// Some voxel data that is generated in the first pass of the compute shader
struct Voxel {
    float density;
    int biomeID;
    int materialID;
};
struct ColorVoxel {
    vec3 color;    
};
// Detail data for the detail spawner
struct Detail {
    // For the first texture
    // Position offset from the current pixel, the offset is actually calculated by dividing this value with 255
    ivec3 position_offset;
    bool spawn;
    // For the second texture
    vec3 rotation;
    float scale;
};
// Pack the density data into two integers
ivec2 pack_density(float s_density) {
    uint density = uint(clamp(s_density + 32767, 0, 65535));
    uint density1 = density >> 8;
    uint density2 = density << 24;
    density2 = density2 >> 24;
    return ivec2(density1, density2);
}
// Unpack the density data to a main float
float unpack_density(uvec2 packed_density) {
    // TODO: AAAAAAAA
    float d1 = uintBitsToFloat(packed_density.x << 24);
    float d2 = uintBitsToFloat(packed_density.y << 16);
    return d1+d2;
}