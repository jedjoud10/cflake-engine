#include <engine/shaders/noises/noise2D.glsl>
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/cellular2D.glsl>
#include <engine/shaders/noises/cellular3D.glsl>

// fBM noise, uses 2D simplex noise
float fbm(vec2 pos, uint octaves, float persistence, float lacunarity) {
    float final = 0.0;
    float scale = 1.0;
    float amplitude = 1.0;

    for(uint i = 0; i < octaves; i++) {
        final += snoise(pos * scale + random2(float(i))) * amplitude;
        scale *= lacunarity;
        amplitude *= persistence;
    }

    return final;
}

// fBM noise, uses 3D simplex noise
float fbm(vec3 pos, uint octaves, float persistence, float lacunarity) {
    float final = 0.0;
    float scale = 1.0;
    float amplitude = 1.0;

    for(uint i = 0; i < octaves; i++) {
        final += snoise(pos * scale + random3(float(i))) * amplitude;
        scale *= lacunarity;
        amplitude *= persistence;
    }

    return final;
}

// fBM noise, uses 2D worley noise
vec2 fbmCellular(vec2 pos, uint octaves, float persistence, float lacunarity) {
    vec2 final = vec2(0.0);
    float scale = 1.0;
    float amplitude = 1.0;

    for(uint i = 0; i < octaves; i++) {
        final += (cellular(pos * scale + random2(float(i)))-vec2(0.5)) * amplitude;
        scale *= lacunarity;
        amplitude *= persistence;
    }

    return final;
}

// fBM noise, uses 3D worley noise
vec2 fbmCellular(vec3 pos, uint octaves, float persistence, float lacunarity) {
    vec2 final = vec2(0.0);
    float scale = 1.0;
    float amplitude = 1.0;

    for(uint i = 0; i < octaves; i++) {
        final += (cellular(pos * scale + random3(float(i)))-vec2(0.5)) * amplitude;
        scale *= lacunarity;
        amplitude *= persistence;
    }
    
    return final;
}