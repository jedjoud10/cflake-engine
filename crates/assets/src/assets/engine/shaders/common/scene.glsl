// UBO that contains the current scene information
layout(set = 0, binding = 1) uniform SceneUniform {
    // Ambient color of the environment
    vec4 ambient_color;
    
    // Sun related parameters
    vec4 sun_direction;
    float sun_strength;
    float sun_size;
} scene;