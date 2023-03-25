#include <engine/shaders/noises/noise2D.glsl>
#include <engine/shaders/noises/noise3D.glsl>

// fBM noise, uses 2D simplex noise
float fbm(vec2 pos, int octaves, float persistence, float lacunarity) {
    float final = 0.0;
    float scale = 1.0;
    float amplitude = 1.0;

    for(int i = 0; i < octaves; i++) {
        final += snoise(pos * scale) * amplitude;
        scale *= lacunarity;
        amplitude *= persistence;
    }

    return final;
}

// fBM noise, uses #D simplex noise
float fbm(vec3 pos, int octaves, float persistence, float lacunarity) {
    float final = 0.0;
    float scale = 1.0;
    float amplitude = 1.0;

    for(int i = 0; i < octaves; i++) {
        final += snoise(pos * scale) * amplitude;
        scale *= lacunarity;
        amplitude *= persistence;
    }

    return final;
}