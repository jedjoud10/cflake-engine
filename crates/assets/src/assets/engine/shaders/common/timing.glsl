// UBO that contains the timing information of the current frame
layout(set = 1, binding = 2) uniform TimingUniform {
    // Number of frames that elapsed
    uint frame_count;

    // Delta time (difference in time between current frame and last frame)
    float delta_time;

    // Number of seconds since startup
    float time_since_startup;
} time;