#version 460 core
#include "engine/shaders/math/conversions.func.glsl"
layout(location = 0) out vec3 color;
uniform sampler2D panorama;
in vec3 l_position;

void main() {
    vec2 uv = sample_spherical_map(normalize(l_position));
    color = texture(panorama, uv).rgb;
}