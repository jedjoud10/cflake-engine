#version 460 core
out vec3 frag;

// Main PBR uniforms
uniform float roughness;
uniform float bumpiness;
//uniform float metallic;
uniform vec3 tint;
uniform sampler2D albedo;
uniform sampler2D normal;
//uniform sampler2D mask;

// Uniforms coming from the camera
uniform vec3 camera;
uniform vec3 forward;

// Uniforms set by the main scene
uniform vec3 light_dir;
uniform vec3 light_color;
uniform float light_strength;

// Data given by the vertex shader
in vec3 m_position;
in vec3 m_normal;
in vec3 m_tangent;
in vec3 m_bitangent;
in vec3 m_color;
in vec2 m_tex_coord;

void main() {
    // Fetch the main albedo/diffuse color
    vec3 diffuse = texture(albedo, m_tex_coord).xyz;

    // Calculate the normal mapped bumpiness
	vec3 bumps = texture(normal, m_tex_coord).xyz * 2.0 - 1.0;
	bumps.xy *= bumpiness;

	// Calculate the world space normals
	mat3 tbn = mat3(
		normalize(m_tangent),
		normalize(m_bitangent),
		normalize(m_normal));
	vec3 normal = normalize(tbn * normalize(bumps));

    // Calculate lighting factor
	float ambient = 0.0;
	float dir_light_value = max(dot(normal, light_dir), 0.0) * light_strength; 
    vec3 light = dir_light_value * light_color;

	// Calculate specular light
	vec3 view = normalize(camera - m_position);
	float spec = pow(max(dot(view, reflect(-light_dir, normal)), 0.0), 32) * light_strength;

	// Combine the factors to make the final color
    frag = (vec3(ambient) + light);
}