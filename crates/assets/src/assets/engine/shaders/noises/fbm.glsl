#include <engine/shaders/noises/noise2D.glsl>
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/cellular2D.glsl>
#include <engine/shaders/noises/cellular3D.glsl>

// fBM noise, uses 2D simplex noise
float fbm(vec2 pos, int octaves, float persistence, float lacunarity) {
    float final = 0.0;
    float scale = 1.0;
    float amplitude = 1.0;
    float normalizer = 0.0;

    for(int i = 0; i < octaves; i++) {
        final += snoise(pos * scale) * amplitude;
        scale *= lacunarity;
        amplitude *= persistence;
        normalizer += pow(persistence, i);
    }

    return final / normalizer;
}

// fBM noise, uses 3D simplex noise
float fbm(vec3 pos, int octaves, float persistence, float lacunarity) {
    float final = 0.0;
    float scale = 1.0;
    float amplitude = 1.0;
    float normalizer = 0.0;

    for(int i = 0; i < octaves; i++) {
        final += snoise(pos * scale) * amplitude;
        scale *= lacunarity;
        amplitude *= persistence;
        normalizer += pow(persistence, i);
    }

    return final / normalizer;
}

// fBM noise, uses 2D worley noise
vec2 fbmCellular(vec2 pos, int octaves, float persistence, float lacunarity) {
    vec2 final = vec2(0.0);
    float scale = 1.0;
    float amplitude = 1.0;
    float normalizer = 0.0;

    for(int i = 0; i < octaves; i++) {
        final += (cellular(pos * scale)-vec2(0.5)) * amplitude;
        scale *= lacunarity;
        amplitude *= persistence;
        normalizer += pow(persistence, i);
    }

    // Dunno if this works since cellular noise isn't rlly in the 0 - 1 range. Divisor *must* be higher
    return final / normalizer;
}

// fBM noise, uses 3D worley noise
vec2 fbmCellular(vec3 pos, int octaves, float persistence, float lacunarity) {
    vec2 final = vec2(0.0);
    float scale = 1.0;
    float amplitude = 1.0;
    float normalizer = 0.0;

    for(int i = 0; i < octaves; i++) {
        final += (cellular(pos * scale)-vec2(0.5)) * amplitude;
        scale *= lacunarity;
        amplitude *= persistence;
        normalizer += pow(persistence, i);
    }

    // Dunno if this works since cellular noise isn't rlly in the 0 - 1 range. Divisor *must* be higher
    return final / normalizer;
}