// Terrain voxel generation push constants
layout(push_constant) uniform PushConstants {
    vec4 offset;
    uint global_chunk_index;
    uint allocation_index;
} parameters;

// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/sdf/common.glsl>
#include <engine/shaders/sdf/operations.glsl>
#include <engine/shaders/noises/fbm.glsl>
    
// Voxel type that contains terrain surface params
struct Voxel {
    float density;
    vec3 color;
};

float smooth_floor(float x) {
    return x - (sin(2 * 3.1415 * x) / (2 * 3.1415));
}

// Main voxel function that will create the shape of the terrain
Voxel voxel(vec3 position) {
    position += parameters.offset.xyz;

    /*
    float density = fbm(position * 0.002, 7, 0.4, 2.1).x * 140;

    //density = mix(density, (fbmCellular(position * 0.001, 10, 0.5, 2.2).x) * 130, snoise(position * 0.01) * 0.5 + 0.5); 

    //float density = opSmoothUnion(fbm(position * 0.03, 3, 0.4, 2.2) * 1 + position.y + 5, position.y, 20);
    //density -= fbmCellular(position * 0.02 + vec3(snoise(position * 0.09) * 0.13), 10, 0.4, 2.2).y * 10;
    return Voxel(density + position.y, vec3(1));
    */
    
    float density = position.y;
    density += snoise(position.xz * 0.01) * 20 - (fbmCellular(position * 0.01, 2, 0.3, 2.2).x) * 130;
    return Voxel(density, vec3(1));

    /*
    float density = opSmoothUnion((1-fbmCellular(position * 0.005 * vec3(1, 0.2, 1), 5, 0.4, 2.1).y) * 150 - 120 + position.y, position.y, 30);
    //density = max(density, -sdSphere(position + vec3(0, 10, 0), 30));
    //density = position.y - 2.00;
    return Voxel(density, vec3(random(parameters.global_chunk_index / 400.0)));
    */

    
    /*
    //TEST 1
    position *= 0.3;
    vec3 col = vec3(156, 63, 12) / 255.0;
    vec3 col1 = vec3(168, 68, 25) / 255.0;
    //vec3 col2 = vec3(255.0, 112.0, 5.0) / 255.0;
    vec3 col3 = vec3(156, 87, 39) / 255.0;
    float fac0 = snoise(position * 0.1 * vec3(0, 3, 0)) * 0.5 + 0.5 + random(position) * 0.8;
    float fac1 = snoise(position * 0.1 * vec3(0.0, 1.2, 0.0)) * 0.5 + 0.5 + random(position) * 0.3;

    col = mix(col, col1, fac0); 
    col = mix(col, col3, fac1);

    float density = (1-fbmCellular(position * 0.02 * vec3(1, 5.0, 1), 8, 0.5, 2.0).x) * 10;
    float d2 = (1-fbmCellular(position * 0.008 * vec3(1, 0.1, 1), 8, 0.3, 2.1).x) * 140;
    d2 = smooth_floor(d2 / 50) * 50;

    d2 += position.y;
    d2 = opSmoothUnion(d2, position.y + 140, 10);
    d2 = opSmoothSubtraction(-d2, position.y + 100, 50);
    density += d2 - 140;
    return Voxel(density, vec3(0));
    */
}