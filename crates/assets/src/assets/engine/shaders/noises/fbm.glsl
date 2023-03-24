#include <engine/shaders/noises/noise2D.glsl>
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/gnoise.glsl>

/*
original_author: Patricio Gonzalez Vivo
description: Fractal Brownian Motion
use: fbm(<vec2> pos)
options:
    FBM_OCTAVES: numbers of octaves. Default is 4.
    FBM_NOISE_FNC(UV): noise function to use Default 'snoise(UV)' (simplex noise)
    FBM_VALUE_INITIAL: initial value. Default is 0.
    FBM_SCALE_SCALAR: scalar. Defualt is 2.
    FBM_AMPLITUD_INITIAL: initial amplitud value. Default is 0.5
    FBM_AMPLITUD_SCALAR: amplitud scalar. Default is 0.5
examples:
    - /shaders/generative_fbm.frag
*/

float FBM_VALUE_INITIAL = 0.0f;

float FBM_SCALE_SCALAR = 2.0f;

float FBM_AMPLITUD_INITIAL = 0.5f;

float FBM_AMPLITUD_SCALAR = 0.5f;

float fbm(vec2 st, int octaves) {
    // Initial values
    float value = FBM_VALUE_INITIAL;
    float amplitud = FBM_AMPLITUD_INITIAL;

    // Loop of octaves
    for (int i = 0; i < octaves; i++) {
        value += amplitud * snoise(st);
        st *= FBM_SCALE_SCALAR;
        amplitud *= FBM_AMPLITUD_SCALAR;
    }
    return value;
}

float fbm(vec3 pos, int octaves) {
    // Initial values
    float value = FBM_VALUE_INITIAL;
    float amplitud = FBM_AMPLITUD_INITIAL;

    // Loop of octaves
    for (int i = 0; i < octaves; i++) {
        value += amplitud * snoise(pos);
        pos *= FBM_SCALE_SCALAR;
        amplitud *= FBM_AMPLITUD_SCALAR;
    }
    return value;
}

float fbm(vec3 p, int octaves, float tileLength) {
    const float persistence = 0.5;
    const float lacunarity = 2.0;

    float amplitude = 0.5;
    float total = 0.0;
    float normalization = 0.0;

    for (int i = 0; i < octaves; ++i) {
        float noiseValue = gnoise(p, tileLength * lacunarity * 0.5) * 0.5 + 0.5;
        total += noiseValue * amplitude;
        normalization += amplitude;
        amplitude *= persistence;
        p = p * lacunarity;
    }

    return total / normalization;
}