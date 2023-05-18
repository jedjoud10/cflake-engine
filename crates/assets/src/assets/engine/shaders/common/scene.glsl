// UBO that contains the current scene information
layout(set = 0, binding = 1) uniform SceneUniform {
    // Sun related parameters
    vec4 sun_direction;
    vec4 sun_color;

    // Ambient color of the environment
    float ambient_color_strength;
    
    // Procedural sun circle parameters
    float sun_circle_strength;
    float sun_circle_size;
    float sun_circle_fade;
} scene;

// Environment texture map
layout(set = 0, binding = 69) uniform textureCube environment_map;
layout(set = 0, binding = 70) uniform sampler environment_map_sampler;