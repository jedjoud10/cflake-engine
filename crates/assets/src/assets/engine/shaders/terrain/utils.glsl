
// Pack all vertex info into a simple uvec4
uvec4 pack_vertex_data(
	uvec3 cell_position,
    vec3 inner_position,
    vec3 normal,
    vec3 color,
    uint material,
    float ao,
    float roughness, 
    float metallic,
) {
    return uvec4(0);
}

// Fetch the packed vertex data (stored in the position attribute) for a single vertex
uvec4 fetch_packed(uint vertex) {
	uint base = indirect.data[draw].base_index;
	uint vertex_offset = indirect.data[draw].vertex_offset;
	uint index = input_triangles.data[gl_PrimitiveID * 3 + base + vertex];
	uvec4 packed = input_vertices.data[index + vertex_offset];
	return packed;
}

// Fetch the vertex info of a vertex
void fetch_vertex_data(
    uvec4 packed,
    out vec3 position,
    out vec3 normal,
    out vec3 color,
    out vec3 mask,
    out uint material,
) {
	vec4 cell_position = unpackUnorm4x8(packed.x) * 255;
    vec4 inner_position = unpackSnorm4x8(packed.y);
    position = (cell_position + inner_position).xyz;
    normal = unpackSnorm4x8(packed.z).xyz;
    color = unpackUnorm4x8(packed.w).xyz;
    mask = vec3(0);
    material = 0;
}