// A final voxel that will be packed, then sent back to the CPU
struct FinalVoxel {
    float density;
    vec3 normal;
    uint material; 
};

// Get the final voxel at a specific position (Second Pass)
FinalVoxel get_final_voxel(vec3 pos, vec3 normal, Voxel voxel) {
    return FinalVoxel(voxel.density, normal, voxel.material);
}

// A packed voxel that is also stored in an array, but we will read it back eventually on the CPU
struct PackedVoxel {
    float density;
    uint x_y_z_material;     
};

// Get the packed voxel at a specific position (Second Pass)
PackedVoxel get_packed_voxel(FinalVoxel voxel) {
    // Pack the data into 2 ints
    uint x_y_z_material = packSnorm4x8(vec4(normalize(voxel.normal).xyz, 0.0));
    x_y_z_material |= voxel.material << 24;
    return PackedVoxel(voxel.density, x_y_z_material);
}

// Flatten a 3D position to an index that is part of a 3D flattened array of axis length "size"
int flatten(ivec3 pc, int size) {
    return pc.x + (pc.y * size) + (pc.z * size * size);
}