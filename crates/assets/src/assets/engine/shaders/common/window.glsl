// UBO that contains the current monitor/window information
layout(set = 0, binding = 3) uniform WindowUniform {
    // Dimensions of the window
    uint width;
    uint height;
} window;