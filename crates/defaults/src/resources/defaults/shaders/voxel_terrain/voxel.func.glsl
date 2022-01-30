// A simple voxel that is stored in an array, in a GPU buffer 
// This voxel struct can contain some arbitrary values related to voxel generation
struct Voxel {
    float density;
    vec3 color;

    // Actual range for this is 65535, since on the CPU we store this as a u16
    uint mat_type;
};

// A final voxel that is also stored in an array, but we will read it back eventually on the CPU
// This final voxel should have only the nessecarry values that we need on a vertex to vertex basis
struct FinalVoxel {
    // Normal { X, Y, Z } and Density components stored in two ints (4bytes each)
    uint density_x;
    uint y_z;     
    // Color { X, Y, Z } and Material type stored in two ints (4bytes each) 
    uint x_y;
    uint z_mat_type;  
};

// Get the voxel at a specific position (First Pass)
Voxel get_voxel(vec3 pos) {
    return Voxel(pos.y + snoise(pos * 0.006 * vec3(1.0, 2.0 + sin(pos.y * 0.2), 1.0)) * 20.0, vec3(sin(pos.x), sin(pos.y-15.21), sin(pos.z+123)), 0);
}

// Get the final voxel at a specific position (Second Pass)
FinalVoxel get_final_voxel(vec3 pos, vec3 normal, Voxel voxel) {
    float density = voxel.density;
    // Pack the data into 2 ints
    uint density_x = packHalf2x16(vec2(density, normal.x));
    uint y_z = packHalf2x16(normal.yz);
    // Pack some more data into two ints
    uint x_y = packHalf2x16(voxel.color.xy);
    uint z_mat_type = packHalf2x16(vec2(voxel.color.z, uintBitsToFloat(voxel.mat_type)));
    return FinalVoxel(density_x, y_z, x_y, z_mat_type);
}