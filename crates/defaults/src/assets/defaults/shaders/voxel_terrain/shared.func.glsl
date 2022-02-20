// A final voxel that will be packed, then sent back to the CPU
struct FinalVoxel {
    float density;
    vec3 normal;
    vec3 color;
    uint material; 
};

// Get the final voxel at a specific position (Second Pass)
FinalVoxel get_final_voxel(vec3 pos, vec3 normal, Voxel voxel) {
    return FinalVoxel(voxel.density, normal, voxel.color.rgb, voxel.material);
}

// A packed voxel that is also stored in an array, but we will read it back eventually on the CPU
struct PackedVoxel {
    uint density_rgbcolor_nothing;
    uint x_y_z_material;     
};

// Get the packed voxel at a specific position (Second Pass)
PackedVoxel get_packed_voxel(FinalVoxel voxel) {
    // Pack the data
    uint x_y_z_material = packSnorm4x8(vec4(normalize(voxel.normal).xyz, 0.0));
    x_y_z_material |= voxel.material << 24;
    uint density_rgbcolor_nothing = packHalf2x16(vec2(voxel.density, 0.0));
    uvec3 color = uvec3(clamp(voxel.color, 0, 1) * 255);
    // 5 6 5
    uint quantanized_color = (color.x / 8) << 11;
    quantanized_color |= (color.y / 4) << 5;
    quantanized_color |= (color.z / 8);
    density_rgbcolor_nothing |= quantanized_color << 16;
    return PackedVoxel(density_rgbcolor_nothing, x_y_z_material);
}

// Flatten a 3D position to an index that is part of a 3D flattened array of axis length "size"
int flatten(ivec3 pc, int size) {
    return pc.x + (pc.y * size) + (pc.z * size * size);
}