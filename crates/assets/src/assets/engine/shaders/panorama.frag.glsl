#version 460 core
layout(location = 0) out vec3 color;
uniform sampler2D panorama;
in vec3 m_position;

// A bit of conversion magic from https://learnopengl.com/PBR/IBL/Diffuse-irradiance
const vec2 invAtan = vec2(0.1591, 0.3183);
vec2 sample_spherical_map(vec3 v)
{
    vec2 uv = vec2(atan(v.z, v.x), asin(v.y));
    uv *= invAtan;
    uv += 0.5;
    return uv;
}

void main() {
    vec2 uv = sample_spherical_map(normalize(local_pos));
    color = texture(panorama, uv).rgb;
}