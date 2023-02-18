// UBO that contains the current scene information
layout(set = 0, binding = 1) uniform SceneUniform {
    // Ambient color of the environment
    vec4 ambient_color;

    // Sun related parameters
    float sun_strength;
    float sun_size;

    // Index that points to the skybox texture used
    int skybox_texture;
} scene;