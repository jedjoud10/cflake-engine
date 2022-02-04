// A final voxel that will be packed, then sent back to the CPU
struct FinalVoxel {
    float density;
    vec3 normal;
    vec3 color;
    uint material; 
};

// Get the final voxel at a specific position (Second Pass)
FinalVoxel get_final_voxel(vec3 pos, vec3 normal, Voxel voxel) {
    return FinalVoxel(voxel.density, normal, voxel.color, voxel.material);
}

// A packed voxel that is also stored in an array, but we will read it back eventually on the CPU
struct PackedVoxel {
    float density;
    // Normal { X, Y, Z } in a single uint
    uint x_y_z_padding;     
    // Color { X, Y, Z } and Material stored in a single uint
    uint x_y_z_material;
    uint nothing;
};

// Get the packed voxel at a specific position (Second Pass)
PackedVoxel get_packed_voxel(FinalVoxel voxel) {
    // Clamp the values first
    voxel.normal = normalize(voxel.normal);
    voxel.color = clamp(voxel.color, 0, 1);
    // Pack the data into 2 ints
    uint x_y_z_padding = packSnorm4x8(vec4(voxel.normal.xyz, 0.0));
    // Pack some more data into two ints
    uint x_y_z_material = packUnorm4x8(vec4(voxel.color.xyz, (float(voxel.material)/255.0)));
    return PackedVoxel(voxel.density, x_y_z_padding, x_y_z_material, 0);
}

// Flatten a 3D position to an index that is part of a 3D flattened array of axis length "size"
int flatten(ivec3 pc, int size) {
    return pc.x + (pc.y * size) + (pc.z * size * size);
}