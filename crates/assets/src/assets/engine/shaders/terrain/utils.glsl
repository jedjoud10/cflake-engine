
// Pack all vertex info into a simple uvec4
uvec4 pack_vertex_data(
	uvec3 cell_position,
    vec3 inner_position,
    vec3 normal,
    vec3 color,
    uint material,
    float ao,
    float roughness, 
    float metallic
) {
    return uvec4(0);
}

// Fetch the vertex info of a vertex
void fetch_vertex_data(
    vec4 packed,
    out vec3 position,
    out vec3 normal,
    out vec3 color,
    out vec3 mask,
    out uint material
) {
	vec4 cell_position_ao = unpackUnorm4x8(floatBitsToUint(packed.x));
    vec4 inner_position = unpackSnorm4x8(floatBitsToUint(packed.y));
    position = (cell_position_ao * 255.0 + inner_position).xyz;
    normal = unpackSnorm4x8(floatBitsToUint(packed.z)).xyz;
    color = unpackUnorm4x8(floatBitsToUint(packed.w)).xyz;
    mask = vec3(cell_position_ao.w, 1, 0);
    material = 0;
}