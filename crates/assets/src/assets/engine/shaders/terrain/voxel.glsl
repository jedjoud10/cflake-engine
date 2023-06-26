// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/sdf/common.glsl>
#include <engine/shaders/sdf/operations.glsl>
#include <engine/shaders/noises/fbm.glsl>

float smooth_floor(float x) {
    return x - (sin(2 * 3.1415 * x) / (2 * 3.1415));
}

float smooth_floor2(float x, float n) {
    return (pow(x, n) - pow((1 - x), n)) * 0.5;
}



vec2 rotate(vec2 v, float a) {
	float s = sin(a);
	float c = cos(a);
	mat2 m = mat2(c, -s, s, c);
	return m * v;
}

// Main voxel function that will create the shape of the terrain
// Negative values represent terrain, positive values represent air
float voxel(vec3 position, out vec3 color, out uint material) {
    position *= 1.;

    // Blend between the two biomes
    float blend = clamp(snoise(position.xz * 0.0001) * 0.5 + 0.5, 0, 1);
    blend = smoothstep(0.0, 1.0, clamp((blend - 0.5) * 6 + 0.5, 0, 1));
       
    // Sand/Dune biome
    float biome1 = 0;
    vec3 color1 = vec3(0);
    uint material2 = 2;

    if (blend != 1.0) {
        vec2 rotated = rotate(position.xz, 3.1415 / 4.0);
        float spikey = snoise(position.xz * 0.0003) * 0.5 + 0.5;
        spikey = clamp((spikey - 0.5) * 1 + 0.5, 0, 1); 
        biome1 = (1 - spikey) * snoise(position.xz * 0.001 + vec2(snoise(position.xz * 0.0002)) * vec2(1.3, 0.2)) * 60;
        biome1 += (1 - spikey) * sin(dot(position.xz, vec2(1, 1)) * 0.01 - 1.202) * 30;
        biome1 += (1 - spikey) * cos(dot(position.xz, vec2(0.2, 2)) * 0.001 + 1.2) * 45;
        float spikey2 = pow(abs(snoise(rotated * vec2(2.3, 0.7) * 0.0013 + vec2(snoise(position * 0.0012)) * 0.2)), 1.2);
        biome1 += spikey * spikey2 * 20;
        biome1 += position.y;
        color1 = (vec3(1 - spikey2) * 0.3 + 0.5 + snoise(position * 0.1) * 0.1) * pow(vec3(255, 188, 133) / 255.0, vec3(2.2));
    }

    // Rocky biome
    float rocky = 0.0;
    vec3 color2 = vec3(0);
    uint material3 = 1;

    if (blend != 0.0) {
        rocky = position.y - fbmCellular(position.xz * 0.001, 8, 0.5, 1.95).x * 930 - 50;
        rocky = opSmoothUnion(position.y, rocky, 400);
        color2 = (snoise(position * vec3(0, 10, 0) * 0.004) * 0.4 + 0.4) * pow(vec3(100.0) / 255.0, vec3(2.2));
        rocky += smooth_floor(position.y / 50) * 20;
    }

    float density = mix(biome1, rocky, blend);
    color = mix(color1, color2, blend);

    if (blend < 0.5) {
        material = material2;
    } else {
        material = material3;
    }
    
    return opIntersection(-density, opUnion(-sdBox(position, vec3(1000)), sdSphere(position, 1200)));
}