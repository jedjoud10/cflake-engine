layout(set = 1, binding = 2) uniform TimingUniform {
    uint frame_count;
    float delta_time;
    float time_since_startup;
} time;