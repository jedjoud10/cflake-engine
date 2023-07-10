#version 460 core

// G-Buffer data write
layout(location = 0) out vec4 gbuffer_albedo;
layout(location = 1) out vec4 gbuffer_normal;
layout(location = 2) out vec4 gbuffer_mask;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;
layout(location = 1) in vec3 m_local_position;
layout(location = 2) in flat uint draw; 

#if defined(flat) && defined(attributes)
layout(location = 3) in flat vec3 m_normal;
layout(location = 4) in flat vec3 m_color;
layout(location = 5) in flat vec3 m_mask;
#elif defined(attributes)
layout(location = 3) in vec3 m_normal;
layout(location = 4) in vec3 m_color;
layout(location = 5) in vec3 m_mask;
#endif

// Used to calculate barycentric coordinates
layout (constant_id = 1) const uint input_vertices_count = 1;
layout (constant_id = 2) const uint input_triangles_count = 1;
layout(std430, set = 2, binding = 1) readonly buffer InputVertices {
    uvec4 data[input_vertices_count];
} input_vertices;
layout(std430, set = 2, binding = 2) readonly buffer InputTriangles {
    uint data[input_triangles_count];
} input_triangles;
struct IndexedIndirectDrawArgs {
    uint vertex_count;
    uint instance_count;
    uint base_index;
    int vertex_offset;
    uint base_instance;
};
layout(std430, set = 2, binding = 3) readonly buffer IndirectBuffer {
    IndexedIndirectDrawArgs data[];
} indirect;

#if defined(submaterials)
// Albedo / diffuse map texture array
layout(set = 0, binding = 8) uniform texture2DArray layered_albedo_map;
layout(set = 0, binding = 9) uniform sampler layered_albedo_map_sampler;

// Normal map texture array
layout(set = 0, binding = 10) uniform texture2DArray layered_normal_map;
layout(set = 0, binding = 11) uniform sampler layered_normal_map_sampler;

// Mask map texture array
layout(set = 0, binding = 12) uniform texture2DArray layered_mask_map;
layout(set = 0, binding = 13) uniform sampler layered_mask_map_sampler;

// Triplanar mapping offset and UV scale
const float offset = 0.0;
const vec2 scale = vec2(0.01) * vec2(-1, -1); 
const float normal_strength = 0.8;

// Get the blending offset to be used internally in the triplanar texture
vec3 get_blend(vec3 normal) {
	normal = abs(normal);
	vec3 weights = (max(normal + offset, 0));
	weights /= weights.x + weights.y + weights.z;
	return weights;
}

vec3 triplanar_albedo(float layer, vec3 normal) {
	vec3 blending = get_blend(normalize(normal));

	// Sample the diffuse texture three times to make the triplanar texture
	vec3 diffusex = texture(sampler2DArray(layered_albedo_map, layered_albedo_map_sampler), vec3(m_position.zy * scale, layer)).xyz * blending.x;
	vec3 diffusey = texture(sampler2DArray(layered_albedo_map, layered_albedo_map_sampler), vec3(m_position.xz * scale, layer)).xyz * blending.y;
	vec3 diffusez = texture(sampler2DArray(layered_albedo_map, layered_albedo_map_sampler), vec3(m_position.xy * scale, layer)).xyz * blending.z;
	vec3 diffuse_final = diffusex + diffusey + diffusez;
	return diffuse_final;
}

vec3 triplanar_mask(float layer, vec3 normal) {
	vec3 blending = get_blend(normalize(normal));

	// Sample the diffuse texture three times to make the triplanar texture
	vec3 diffusex = texture(sampler2DArray(layered_mask_map, layered_mask_map_sampler), vec3(m_position.zy * scale, layer)).xyz * blending.x;
	vec3 diffusey = texture(sampler2DArray(layered_mask_map, layered_mask_map_sampler), vec3(m_position.xz * scale, layer)).xyz * blending.y;
	vec3 diffusez = texture(sampler2DArray(layered_mask_map, layered_mask_map_sampler), vec3(m_position.xy * scale, layer)).xyz * blending.z;
	vec3 diffuse_final = diffusex + diffusey + diffusez;
	return diffuse_final;
}

// https://bgolus.medium.com/normal-mapping-for-a-triplanar-shader-10bf39dca05a
vec3 triplanar_normal(float layer, vec3 normal) {
	vec3 wnormal =  normalize(normal);
	vec3 blending = get_blend(wnormal);

	// Do the same for the normal map
	vec3 normalx = texture(sampler2DArray(layered_normal_map, layered_normal_map_sampler), vec3(m_position.zy * scale, layer)).xyz * 2 - 1;
	vec3 normaly = texture(sampler2DArray(layered_normal_map, layered_normal_map_sampler), vec3(m_position.xz * scale, layer)).xyz * 2 - 1;
	vec3 normalz = texture(sampler2DArray(layered_normal_map, layered_normal_map_sampler), vec3(m_position.xy * scale, layer)).xyz * 2 - 1;
	normalx = vec3(vec2(normalx.x, normalx.y) * normal_strength + wnormal.zy, wnormal.x) * blending.x;
	normaly = vec3(vec2(normaly.x, normaly.y) * normal_strength + wnormal.xz, wnormal.y) * blending.y;
	normalz = vec3(vec2(normalz.x, normalz.y) * normal_strength + wnormal.xy, wnormal.z) * blending.z;
	vec3 normal_final = normalize(normalx.zyx + normaly.xzy + normalz.xyz);
	return normal_final;
}
#endif

void main() {
	// Fetch packed vertex data
	uvec4 p0 = fetch_packed(0);
	uvec4 p1 = fetch_packed(1);
	uvec4 p2 = fetch_packed(2);

	// Fetch chunk local positions of vertices
	vec3 v0 = fetch_vertex_position(p0.xy);
	vec3 v1 = fetch_vertex_position(p1.xy);
	vec3 v2 = fetch_vertex_position(p2.xy);

	// Output variables
	vec3 albedo = vec3(0);
	vec3 normal = vec3(0);
	vec3 mask = vec3(0);

	// Either handle attributes (flat or non-flat) for or averaged out attributes
	#if defined(attributes)	

	albedo = m_color;
	normal = normalize(m_normal);
	mask = m_mask;

	#else
	
	// Fetch colors of vertices
	vec3 c0 = fetch_vertex_colors(p0.w);
	vec3 c1 = fetch_vertex_colors(p1.w);
	vec3 c2 = fetch_vertex_colors(p2.w);

	// Fetch normals of vertices
	vec3 n0 = fetch_vertex_normal(p0.z);
	vec3 n1 = fetch_vertex_normal(p1.z);
	vec3 n2 = fetch_vertex_normal(p2.z);

	// Fetch mask of vertices
	vec3 n0 = fetch_vertex_mask(p0);
	vec3 n1 = fetch_vertex_mask(p1);
	vec3 n2 = fetch_vertex_mask(p2);


	albedo = (c0 + c1 + c2) / 3.0;
	normal = normalize(n0 + n1 + n2);
	mask = vec3(1, 0.1, 0);

	#endif

	// OVERWRITES THE NORMALS IN CASE OF DERIVED NORMALS
	// (compiler should get rid of this so it's ok)
	#if defined(derived)
	normal = normalize(cross(dFdy(m_position), dFdx(m_position)));
	#endif

	gbuffer_albedo = vec4(albedo, 1);
	gbuffer_normal = vec4(normal, 0);
	gbuffer_mask = vec4(mask, 0);

	/*
	vec4 v0 = fetch_vertex_position_and_extra(0);
	vec4 v1 = fetch_vertex_position_and_extra(1);
	vec4 v2 = fetch_vertex_position_and_extra(2);
	float i0 = unpackUnorm4x8(floatBitsToUint(v0.w)).w * 255.0;
	float i1 = unpackUnorm4x8(floatBitsToUint(v1.w)).w * 255.0;
	float i2 = unpackUnorm4x8(floatBitsToUint(v2.w)).w * 255.0;
	
	vec3 c0 = unpackUnorm4x8(floatBitsToUint(v0.w)).xyz;
	vec3 c1 = unpackUnorm4x8(floatBitsToUint(v1.w)).xyz;
	vec3 c2 = unpackUnorm4x8(floatBitsToUint(v2.w)).xyz;

	vec3 albedo = (c0 + c1 + c2) * 0.3333;
	vec3 mask = vec3(1, 1, 0);
	vec3 normal = surface_normal;

	gbuffer_position = vec4(m_position, 0);
	gbuffer_albedo = vec4(albedo, 1);
	gbuffer_normal = vec4(normal, 0);
	gbuffer_mask = vec4(mask * vec3(1, 3, 1), 0);
	*/

	/*
	if ((i0 == i1) && (i2 == i1)) {
		albedo = triplanar_albedo(i0, surface_normal);
		mask = triplanar_mask(i0, surface_normal);
		normal = triplanar_normal(i0, surface_normal);
	} else {
		vec3 a0 = triplanar_albedo(i0, surface_normal);
		vec3 m0 = triplanar_mask(i0, surface_normal);
		vec3 n0 = triplanar_normal(i0, surface_normal);

		vec3 a1 = triplanar_albedo(i1, surface_normal);
		vec3 m1 = triplanar_mask(i1, surface_normal);
		vec3 n1 = triplanar_normal(i1, surface_normal);

		vec3 a2 = triplanar_albedo(i2, surface_normal);
		vec3 m2 = triplanar_mask(i2, surface_normal);
		vec3 n2 = triplanar_normal(i2, surface_normal);

		// https://gamedev.stackexchange.com/questions/23743/whats-the-most-efficient-way-to-find-barycentric-coordinates
		// TODO: Optimize?
		vec3 vect0 = v1.xyz - v0.xyz;
		vec3 vect1 = v2.xyz - v0.xyz;
		vec3 vect2 = m_local_position - v0.xyz;
		float d00 = dot(vect0, vect0);
		float d01 = dot(vect0, vect1);
		float d11 = dot(vect1, vect1);
		float d20 = dot(vect2, vect0);
		float d21 = dot(vect2, vect1);
		float denom = d00 * d11 - d01 * d01;
		float v = (d11 * d20 - d01 * d21) / denom;
		float w = (d00 * d21 - d01 * d20) / denom;
		float u = 1.0f - v - w;

		float w0 = u;
		float w1 = v;
		float w2 = w;

		albedo = a0 * w0 + a1 * w1 + a2 * w2;
		mask = m0 * w0 + m1 * w1 + m2 * w2;
		normal = normalize(n0 * w0 + n1 * w1 + n2 * w2);
	}

	gbuffer_position = vec4(m_position, 0);
	gbuffer_albedo = vec4(albedo * m_color, 1);
	gbuffer_normal = vec4(normal, 0);
	gbuffer_mask = vec4(mask * vec3(1, 3, 1), 0);
	*/
}