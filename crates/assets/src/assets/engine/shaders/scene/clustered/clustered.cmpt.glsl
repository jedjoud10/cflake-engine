#version 460 core
#include "engine/shaders/scene/clustered/clustered.func.glsl"
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(rgba32f, binding = 0) uniform image2D image;

void main() {
}