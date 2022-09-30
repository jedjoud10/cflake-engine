#version 460 core
#include "engine/shaders/math/sequences.func.glsl"
layout(location = 0) out vec3 color;
uniform sampler2D panorama;
uniform float roughness;
in vec3 l_position;

// A bit of conversion magic from https://learnopengl.com/PBR/IBL/Diffuse-irradiance
const vec2 invAtan = vec2(0.1591, 0.3183);
vec2 sample_spherical_map(vec3 v)
{
    vec2 uv = vec2(atan(v.z, v.x), asin(v.y));
    uv *= invAtan;
    uv += 0.5;
    return uv;
}

// Get orthonormal basis from surface normal
// https://graphics.pixar.com/library/OrthonormalB/paper.pdf
// https://www.shadertoy.com/view/3tlBW7
void pixarONB(vec3 n, out vec3 b1, out vec3 b2) {
	float sign_ = sign(n.z);
	float a = -1.0 / (sign_ + n.z);
	float b = n.x * n.y * a;
	b1 = vec3(1.0 + sign_ * n.x * n.x * a, sign_ * b, -sign_ * n.x);
	b2 = vec3(b, sign_ + n.y * n.y * a, -n.y);
}

// https://learnopengl.com/PBR/IBL/Specular-IBL
void main() {
    // Create the resulting variables used for convolution
    vec3 dir = normalize(l_position);
    vec3 convoluted = vec3(0.0);     
    float weight = 0.0;   
    const float PI = 3.14159265359;
    const uint SAMPLE_COUNT = 1024;
    vec2 offset = vec2(0.0);

    // GGX Importance sampling moment
    // To be honest, I have no fucking idea what this does bro I am stoopid
    for(uint i = 0; i < SAMPLE_COUNT; ++i)
    {
        offset = weyl(offset, SAMPLE_COUNT);

        float a = roughness*roughness;	
        float phi = 2.0 * PI * offset.x;
        float cos_theta = sqrt((1.0 - offset.y) / (1.0 + (a*a - 1.0) * offset.y));
        float sin_theta = sqrt(1.0 - cos_theta*cos_theta);
    
        // From spherical coordinates to cartesian coordinates
        vec3 halfway;
        halfway.x = cos(phi) * sin_theta;
        halfway.y = sin(phi) * sin_theta;
        halfway.z = cos_theta;
    
        // No idea what this does but alright
        vec3 tangent;
        vec3 bitangent;
        pixarONB(dir, tangent, bitangent);
    
        // Create proper sampling vectors for the texture
        vec3 sampleVec = tangent * halfway.x + bitangent * halfway.y + dir * halfway.z;
        vec3 light = normalize(2.0 * dot(dir, sampleVec) * sampleVec - dir);
        vec2 uv = sample_spherical_map(light);

        float result = max(dot(dir, light), 1e-5);
        if(result > 0.0)
        {   
            convoluted += texture(panorama, uv).rgb * result;
            weight += result;
        }
    }

    // Return the specular IBL diffuse lighting
    color = convoluted / weight;
}
