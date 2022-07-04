#version 460 core
out vec4 frag;

// Main PBR uniforms
uniform float roughness;
uniform float bumpiness;
//uniform float metallic;
//uniform vec3 tint;
uniform sampler2D albedo;
uniform sampler2D normal;
//uniform sampler2D mask;

// Uniforms coming from the camera
uniform vec3 camera;
uniform vec3 forward;

// Uniforms set by the main scene
uniform vec3 light;

// Data given by the vertex shader
in vec3 m_position;
in vec3 m_normal;
in vec3 m_tangent;
in vec3 m_bitangent;
in vec3 m_color;
in vec2 m_tex_coord_0;

void main() {
    // Fetch the main albedo/diffuse color
    vec3 diffuse = texture(albedo, m_tex_coord_0).xyz;

    // Calculate the normal mapped bumpiness
	vec3 bumps = texture(normal, m_tex_coord_0).xyz * 2.0 - 1.0;
	bumps.xy *= bumpiness;

	// Calculate the world space normals (TBN matrix)
	mat3 tbn = mat3(
		normalize(m_tangent),
		normalize(m_bitangent),
		normalize(m_normal));
	vec3 normal = normalize(tbn * normalize(bumps));

    // Calculate lighting factor
    float light = dot(normal, vec3(0, 1, 0));

    // This sets the color for the current fragment
    frag = vec4(light * diffuse, 0.0);
}