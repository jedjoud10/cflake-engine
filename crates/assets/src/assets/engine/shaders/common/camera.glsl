// Camera, scene, and shadowmap shared objects
layout(set = 0, binding = 0) uniform CameraUniform {
    // Projection & view matrix
    mat4 projection;
    mat4 view;

    // Position of the camera and it's directions
    vec4 position;
    vec4 forward;
    vec4 right;
    vec4 up;

    // Near, far, orizontal FOV
    vec4 near_far_vfov_;
    vec4 _a;
    vec4 _b;
    vec4 _c;
} camera;