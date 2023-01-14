layout(set = 1, binding = 0) uniform CameraUniform {
    mat4 projection;
    mat4 inverse_projection;
    mat4 view;
    mat4 inverse_view;
    vec3 position;
    vec3 forward;
    vec3 right;
} camera;
