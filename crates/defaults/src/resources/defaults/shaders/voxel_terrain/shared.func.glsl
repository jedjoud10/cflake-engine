// A final voxel that will be packed, then sent back to the CPU
struct FinalVoxel {
    float density;
    vec3 normal;
    vec3 color;
    float hardness; 
};

// Get the final voxel at a specific position (Second Pass)
FinalVoxel get_final_voxel(vec3 pos, vec3 normal, Voxel voxel) {
    return FinalVoxel(voxel.density, normal, voxel.color, voxel.hardness);
}

// A packed voxel that is also stored in an array, but we will read it back eventually on the CPU
struct PackedVoxel {
    // Normal { X, Y, Z } and Density components stored in two ints (4bytes each)
    uint density_x;
    uint y_z;     
    // Color { X, Y, Z } and Hardness stored in a single uint
    uint x_y_z_hardness;
    uint nothing;
};

// Get the packed voxel at a specific position (Second Pass)
PackedVoxel get_packed_voxel(FinalVoxel voxel) {
    // Clamp the values first
    voxel.normal = normalize(voxel.normal);
    voxel.color = clamp(voxel.color, 0, 1);
    // Pack the data into 2 ints
    uint density_x = packHalf2x16(vec2(voxel.density, voxel.normal.x));
    uint y_z = packHalf2x16(voxel.normal.yz);
    // Pack some more data into two ints
    uint x_y_z_hardness = packUnorm4x8(vec4(voxel.color.xyz, voxel.hardness));
    return PackedVoxel(density_x, y_z, x_y_z_hardness, 0);
}

// Flatten a 3D position to an index that is part of a 3D flattened array of axis length "size"
int flatten(ivec3 pc, int size) {
    return pc.x + (pc.y * size) + (pc.z * size * size);
}