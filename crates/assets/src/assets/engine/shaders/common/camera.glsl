
layout(set = 0, binding = 0, std140) uniform CameraUniform {
    // Projection & inv projection matrix
    mat4 projection;
    mat4 inverse_projection;

    // View & inv view matrix
    mat4 view;
    mat4 inverse_view;

    /*
    // Position of the camera
    vec4 position;

    // Direction vectors
    vec4 forward;
    vec4 right;
    vec4 up;
    */
} camera;
