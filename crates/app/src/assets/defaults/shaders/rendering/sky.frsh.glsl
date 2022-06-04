#version 460 core

// Pixel color
out vec4 color;

// This should be normalized
in vec3 m_position;

void main() {
    // Fetch the cubemap texture
    vec3 normal = normalize(m_position);
    color = vec4(normal, 0);
}