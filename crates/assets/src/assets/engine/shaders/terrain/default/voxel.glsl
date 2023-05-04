// Terrain voxel generation push constants
layout(push_constant) uniform PushConstants {
    vec4 offset;
    float scale;
} parameters;

// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/sdf/common.glsl>
#include <engine/shaders/sdf/operations.glsl>
#include <engine/shaders/noises/fbm.glsl>

    float smooth_floor(float x) {
        return x - (sin(2 * 3.1415 * x) / (2 * 3.1415));
    }
    
// A voxel is a 3D pixel in the world that can contain multiple parameters
// Each voxel contains a "density". 
// Density allows us to represent either full terrain or air, and everything in between
// Main voxel function that will create the shape of the terrain
float voxel(vec3 position) {
    position *= parameters.scale;
    position += parameters.offset.xyz;
    return opSmoothUnion((1-fbmCellular(position * 0.002, 7, 0.4, 2.3).y) * 1050 + position.y, position.y + 800, 10);

    /*

    */
    //return 1;
    //return min(position.y, sdBox(position - vec3(0, 2, 0), vec3(1)));
    /*
    float density = position.y + (1-fbmCellular(position * 0.008 * vec3(1, 3, 1), 5, 0.3, 2.1).y) * 120 + snoise(position * 0.001) * 1000;
    return density;
    */
    /*
    position *= 0.2;
    //float density = snoise(position * 0.03) * 50;
    float density1 = (1-fbmCellular(position * 0.01 * vec3(1, 2, 1), 10, 0.4, 2.0).y) * 20;
    float density2 = opSmoothUnion(-erosion(position.xz * 0.04, 0.112).x * 420 + position.y + 200, position.y, 40) + 5;

    float density = mix(density1, density2, clamp(snoise(position.xz * 0.003) * 0.5 + 0.5, 0, 1)) + position.y;
    float randomized = random(position) * 0.00;
    
    //float randomized = 0.0;
    //float density = position.y + (1-fbmCellular(position * 0.01 * vec3(1, 0.5, 1), 3, 0.5, 2.0).x) * 20;

    // Create a voxel and return it
    return Voxel(density + randomized, vec3(1.0));
    */

    /*
    float density = fbm(position * 0.002, 7, 0.4, 2.1).x * 140;
    return Voxel(density + position.y, 0);
    */
    

    /*

    //density = mix(density, (fbmCellular(position * 0.001, 10, 0.5, 2.2).x) * 130, snoise(position * 0.01) * 0.5 + 0.5); 

    //float density = opSmoothUnion(fbm(position * 0.03, 3, 0.4, 2.2) * 1 + position.y + 5, position.y, 20);
    //density -= fbmCellular(position * 0.02 + vec3(snoise(position * 0.09) * 0.13), 10, 0.4, 2.2).y * 10;
    return Voxel(density + position.y, 0);
    */

    /*
    float density = position.y;
    density += snoise(position.xz * 0.01) * 20 - (fbmCellular(position * 0.01, 2, 0.3, 2.2).x) * 130;
    return Voxel(density, vec3(1));
    */
    
    /*
    float first = (1-fbmCellular(position * 0.004 * vec3(1, 4, 1), 7, 0.6, 1.8).x) * 100.0 - 50;
    float second = -erosion(position.xz * 0.04, 0.1).x * 140;
    float density = position.y + mix(first, second, clamp((snoise(position * 0.001)) * 0.5 + 0.5, 0, 1));
    return density;
    */

    /*
    //TEST 1
    


    position *= 0.5;

    float density = 0.0;
    float d2 = (1-fbmCellular(position * 0.008 * vec3(1, 0.1, 1), 8, 0.3, 2.1).x) * 140;
    d2 = smooth_floor(d2 / 50) * 50;

    d2 += position.y;
    d2 = opSmoothUnion(d2, position.y + 140, 10);
    d2 = opSmoothSubtraction(-d2, position.y + 100, 50);
    density += d2 - 140;
    density = opUnion(density, sdSphere(position, 20));
    return density;

    /*
    position *= 0.1;

    float density = (1-fbmCellular(position * 0.02 * vec3(1, 5.0, 1), 8, 0.5, 2.0).x) * 10;
    float d2 = (1-fbmCellular(position * 0.008 * vec3(1, 0.1, 1), 8, 0.3, 2.1).x) * 140;
    d2 = smooth_floor(d2 / 50) * 50;

    d2 += position.y;
    d2 = opSmoothUnion(d2, position.y + 140, 10);
    d2 = opSmoothSubtraction(-d2, position.y + 100, 50);
    density += d2 - 140;
    return density;
    */

    /*
    position *= 0.1;

    float density = (1-fbmCellular(position * 0.02 * vec3(1, 5.0, 1), 8, 0.5, 2.0).x) * 10;
    float d2 = (1-fbmCellular(position * 0.008 * vec3(1, 0.1, 1), 8, 0.3, 2.1).x) * 140;
    d2 = smooth_floor(d2 / 50) * 50;

    d2 += position.y;
    d2 = opSmoothUnion(d2, position.y + 140, 10);
    d2 = opSmoothSubtraction(-d2, position.y + 100, 50);
    density += d2 - 140;
    return density;
    */
}
