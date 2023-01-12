// Camera matrices
layout(set = 0, binding = 0) uniform CameraUniform {
    mat4 camera_projection;
    mat4 camera_inverse_projection;
    mat4 camera_view;
    mat4 camera_inverse_view;
} camera;

// Timing data
layout(set = 0, binding = 1) uniform TimingUniform {
    uint frame_count;
    float delta_time;
    float time_since_startup;
} time;

// Scene data
layout(set = 0, binding = 2) uniform SceneUniform {
    vec4 ambient_color;
    float sun_strength;
    float sun_size;
} scene;