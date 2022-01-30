// A packed voxel that is also stored in an array, but we will read it back eventually on the CPU
struct PackedVoxel {
    // Normal { X, Y, Z } and Density components stored in two ints (4bytes each)
    uint density_x;
    uint y_z;     
    // Color { X, Y, Z } and Material type stored in two ints (4bytes each) 
    uint x_y;
    uint z_mat_type;  
};

// Get the packed voxel at a specific position (Second Pass)
PackedVoxel get_packed_voxel(FinalVoxel voxel) {
    // Pack the data into 2 ints
    uint density_x = packHalf2x16(vec2(voxel.density, voxel.normal.x));
    uint y_z = packHalf2x16(voxel.normal.yz);
    // Pack some more data into two ints
    uint x_y = packHalf2x16(voxel.color.xy);
    uint z_mat_type = packHalf2x16(vec2(voxel.color.z, uintBitsToFloat(voxel.mat_type)));
    return PackedVoxel(density_x, y_z, x_y, z_mat_type);
}