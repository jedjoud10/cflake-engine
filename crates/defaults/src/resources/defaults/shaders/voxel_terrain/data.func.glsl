// A base voxel that is generated on the first pass
struct BaseVoxel {
    float density; 
    float atm_pressure;
    float temperature;
    float humidity;    
}
// The material voxel can contain the type of material and some tint for a specific voxel.
// This is generated on the second pass
struct MaterialVoxel {
    int diffuse;
    int normal_map;
    vec4 tint;
    vec4 normal;
};

// Generate the base voxel at the specified location
void get_voxel(vec3 pos, out BaseVoxel base) {
    base.density = pos.y + sin(pos.x / 10) * 10.0;
    base.atm_pressure = 0;
    base.temperatue = 0;
    base.humidity = 0;
}

// Generate the material voxel
void get_material(vec3 pos, BaseVoxel base, out MaterialVoxel material) {
    material.diffuse = 0;
    material.normal_map = 0;
    material.tint = vec4(0, 0, 0, 0);
    material.normal = vec4(0, 0, 0, 0);
}