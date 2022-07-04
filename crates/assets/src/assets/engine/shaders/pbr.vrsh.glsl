#version 460 core

// Main attribute set vertex attributes
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 tangent;
layout(location = 3) in vec3 color;
layout(location = 4) in vec2 tex_coord_0;

// Transformation / projection matrices
uniform mat4 _view_matrix;
uniform mat4 _proj_matrix;
uniform mat4 _world_matrix;

// Data to give to the fragment shader
out vec3 m_position;
out vec3 m_normal;
out vec3 m_tangent;
out vec3 m_bitangent;
out vec3 m_color;
out vec2 m_tex_coord_0;
void main()
{
    // Model space -> World space -> Clip space
    vec4 world_pos = _world_matrix * vec4(position, 1);
    vec4 projected = (_proj_matrix * _view_matrix) * world_pos; 
    gl_Position = projected;

    // Set the output variables
    m_position = world_pos.xyz;
    m_normal = (_world_matrix * vec4(normal, 0)).xyz;
    m_tangent = (_world_matrix * vec4(tangent.xyz, 0)).xyz;
    m_tex_coord_0 = tex_coord_0;

    // Calculate world space bitangent
	vec3 bitangent = cross(normalize(m_normal), normalize(tangent.xyz)) * tangent.w;
	m_bitangent = normalize((_world_matrix * vec4(bitangent, 0.0)).xyz);     
}