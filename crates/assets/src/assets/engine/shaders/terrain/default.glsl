// Terrain voxel generation push constants
layout(push_constant) uniform PushConstants {
    vec4 offset;
    float time;
    vec3 _padding;
} parameters;

// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/fbm.glsl>

// Main density function that will create the shape of the terrain
float density(vec3 position) {
    position += parameters.offset.xyz - vec3(0, 1, 0);
    float density = position.y;
    //float density = position.y;
    //density = min(density, position.y);
    //density = min(density, position.x);
    //float density = position.y - 100;
    //position *= 0.06f;
    density += snoise(position * 0.03) * 10.0 + random(position*0.001)*0.5;
    //density += min((1-fbmCellular(position * 0.003 * vec3(1, 2, 1), 8, 0.5f, 2.0f).x) * 560.0 - 340, position.y+20);
    return density;
}