layout(set = 0, binding = 0) uniform CameraUniform {
    // Projection & inv projection matrix
    mat4 projection;
    mat4 inverse_projection;

    // View & inv view matrix
    mat4 view;
    mat4 inverse_view;

    // Position of the camera and it's directions
    vec4 position;
    vec4 forward;
    vec4 right;
    vec4 up;
} camera;
