#version 460 core
#include "engine/shaders/math/sequences.func.glsl"
#include "engine/shaders/math/conversions.func.glsl"
#include "engine/shaders/scene/pbr/models.func.glsl"
layout(location = 0) out vec3 color;
uniform samplerCube cubemap;
uniform float roughness;
uniform uint source_face_resolution;
in vec3 l_position;

// Narkowicz 2015, "ACES Filmic Tone Mapping Curve"
vec3 aces(vec3 x) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

// https://learnopengl.com/PBR/IBL/Specular-IBL
void main() {
    // Create the resulting variables used for convolution
    vec3 dir = normalize(l_position);
    vec3 convoluted = vec3(0.0);     
    float weight = 0.0;   
    const uint SAMPLE_COUNT = 1024;

    // GGX Importance sampling moment
    // To be honest, I have no fucking idea what this does bro I am stoopid
    for(uint i = 0; i < SAMPLE_COUNT; i++)
    {
        vec2 offset = hammersley(i, SAMPLE_COUNT);

        float a = roughness*roughness;	
        float phi = 2.0 * PI * offset.x;
        float cos_theta = sqrt((1.0 - offset.y) / (1.0 + (a*a - 1.0) * offset.y));
        float sin_theta = sqrt(1.0 - cos_theta*cos_theta);
    
        // From spherical coordinates to cartesian coordinates
        vec3 h;
        h.x = cos(phi) * sin_theta;
        h.y = sin(phi) * sin_theta;
        h.z = cos_theta;
    
        vec3 up        = abs(dir.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
        vec3 tangent   = normalize(cross(up, dir));
        vec3 bitangent = cross(dir, tangent);
    
        // Create proper sampling vectors for the texture
        vec3 sampleVec = normalize(tangent * h.x + bitangent * h.y + dir * h.z);
        vec3 light = normalize(2.0 * dot(dir, sampleVec) * sampleVec - dir);
        

        float result = max(dot(dir, light), 0.0);
        if(result > 0.0) {   
            // Fetch the result of the texture using a specific mip level
            float hdotv = dot(sampleVec, dir);
            float ndoth = dot(dir, sampleVec);
            float ndf = ndf(roughness, dir, sampleVec);
            float pdf = (ndf * ndoth / (4.0 * hdotv)) + 0.0001; 

            float resolution = source_face_resolution; // resolution of source cubemap (per face)
            float sa_texel  = 4.0 * PI / (6.0 * resolution * resolution);
            float sa_sample = 1.0 / (float(SAMPLE_COUNT) * pdf + 0.0001);

            float level = roughness == 0.0 ? 0.0 : 0.5 * log2(sa_sample / sa_texel); 
            convoluted += textureLod(cubemap, light, level).rgb * result;
            weight += result;
        }
    }

    // Return the specular IBL diffuse lighting
    color = convoluted / weight;
}
