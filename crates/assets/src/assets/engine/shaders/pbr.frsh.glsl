#version 460 core
out vec4 frag;

// Main PBR uniforms
uniform float _roughness;
uniform float _bumpiness;
//uniform float _metallic;
//uniform vec3 _tint;
uniform sampler2D _albedo;
uniform sampler2D _normal;
//uniform sampler2D _mask;

// Data given by the vertex shader
in vec3 m_position;
in vec3 m_normal;
in vec3 m_tangent;
in vec3 m_bitangent;
in vec3 m_color;
in vec2 m_tex_coord_0;

void main() {

    // Calculate the normal mapped bumpiness
	vec3 bumps = texture(_normal, m_tex_coord_0).xyz * 2.0 - 1.0;
	bumps.xy *= _bumpiness;

	// Calculate the world space normals (TBN matrix)
	mat3 tbn = mat3(
		normalize(m_tangent),
		normalize(m_bitangent),
		normalize(m_normal));
	vec3 normal = normalize(tbn * normalize(bumps));

    // This sets the color for the current fragment
    frag = vec4(m_tex_coord_0, 0, 0);
}