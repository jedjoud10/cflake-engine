// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/sdf/common.glsl>
#include <engine/shaders/sdf/operations.glsl>
#include <engine/shaders/noises/fbm.glsl>

float smooth_floor(float x) {
    return x - (sin(2 * 3.1415 * x) / (2 * 3.1415));
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
    /*
    vec2 rotated = rotate(position.xz, 3.1415 / 4.0);
    float spikey = snoise(position.xz * 0.0003) * 0.5 + 0.5;
    spikey = clamp((spikey - 0.5) * 1 + 0.5, 0, 1); 
    float biome1 = (1 - spikey) * snoise(position.xz * 0.001 + vec2(snoise(position.xz * 0.0002)) * vec2(1.3, 0.2)) * 30;
    float spikey2 = pow(abs(snoise(rotated * vec2(2.3, 0.7) * 0.0013 + vec2(snoise(position * 0.0012)) * 0.2)), 1.2);
    biome1 += spikey * spikey2 * 10;
    color = mix(vec3(1), vec3(.6), spikey2 * spikey) * (snoise(position) * 0.2 + 0.6);
    return position.y + biome1;
    */

    /*
    position *= 2.0;

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
        biome1 += spikey * spikey2 * 40;
        biome1 += position.y;
        color1 = clamp(vec3(pow(1 - (spikey2 * spikey), 1)), vec3(0), vec3(1)) * 0.7 + 0.8;
        color1 *= (snoise(position * 0.1) * 0.1 + 0.9) * pow(vec3(255, 188, 133) / 255.0, vec3(2.2));
    }

    // Rocky biome
    float rocky = 0.0;
    vec3 color2 = vec3(0);
    uint material3 = 1;

    if (blend != 0.0) {
        vec3 offseting = vec3(snoise(position * 0.001) * 0.1);
        rocky = -fbmCellular(position * 0.001 * vec3(1, 1.2, 1), 7, 0.5, 1.95).y * 330 - 50;
        rocky += fbmCellular(position.xz * 0.01, 2, 0.5, 1.95).x * 10;
        rocky = opSmoothUnion(position.y, rocky, 400);
        color2 = (snoise(position * vec3(0, 10, 0) * 0.004) * 0.4 + 0.4) * pow(vec3(100.0) / 255.0, vec3(2.2));
        //rocky += position.y;
        rocky += smooth_floor(position.y / 200) * 200;
    }

    float density = mix(biome1, rocky, blend);
    color = mix(color1, color2, blend);

    if (blend < 0.5) {
        material = material2;
    } else {
        material = material3;
    }
    
    // , opUnion(-sdBox(position, vec3(1000)), sdSphere(position, 1200)))
    //return ;
    float pyramide = sdPyramid(position * 0.002 + vec3(0, 0.112, 0), 0.6) * 240;
    pyramide = pyramide + (floor(position.y / 20) - position.y / 20) * 6;

    if (pyramide < 5.0) {
        color = (snoise(position * 0.1) * 0.1 + 0.9) * pow(vec3(255, 188, 133) / 255.0, vec3(2.2));
    }

    return density;
    */

    // Rocky biome 2
    float value = fbm(position.xz * 0.002, 8, 0.5, 2.1) * 100;
    value = opSmoothUnion(value, 0, 100);
    float cel = fbmCellular(position * 0.003 * vec3(1, 2, 1), 4, 0.5, 1.8).x * 50.0;
    value -= cel;
    color = (snoise(position * vec3(0, 10, 0) * 0.004) * 0.4 + 0.4) * pow(vec3(100.0) / 255.0, vec3(2.2));
    color *= clamp(-cel / 20 + 0.9, 0, 1) + 0.2;
    value = smooth_floor(value / 50) * 50;
    value = opSmoothIntersection(value, position.y - 250, 40);
    float island = mix(position.y, 0, snoise(position * 0.001));
    return island + 10 + value;

    /*
    vec2 rotated = rotate(position.xz, 3.1415 / 4.0);
    float spikey = snoise(position.xz * 0.0003) * 0.5 + 0.5;
    spikey = clamp((spikey - 0.5) * 1 + 0.5, 0, 1); 
    float biome1 = (1 - spikey) * snoise(position.xz * 0.001 + vec2(snoise(position.xz * 0.0002)) * vec2(1.3, 0.2)) * 30;
    float spikey2 = pow(abs(snoise(rotated * vec2(2.3, 0.7) * 0.0013 + vec2(snoise(position * 0.0012)) * 0.2)), 1.2);
    biome1 += spikey * spikey2 * 10;
    color = mix(vec3(1), vec3(.6), spikey2 * spikey);
    return position.y + biome1;
    */
}