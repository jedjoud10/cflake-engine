layout(set = 1, binding = 0) uniform CameraUniform {
    // Projection & inv projection matrix
    mat4 projection;
    mat4 inverse_projection;

    // View & inv view matrix
    mat4 view;
    mat4 inverse_view;

    // Position of the camera
    vec3 position;

    // Difrection vectors
    vec3 forward;
    vec3 right;
    vec3 up;
} camera;
