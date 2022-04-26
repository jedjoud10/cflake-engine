// Data just for models and their information
uniform mat4 _model_matrix;

// Data that will be passed from the vertex shader to the fragment shader
struct VertexData {
    vec3 position;
    vec3 normal;
    vec3 tangent;
    vec3 bitangent;
    vec2 uv;
    vec3 color;
};