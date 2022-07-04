#version 460 core
out vec4 frag_color;
//uniform float _roughness;
//uniform float _metallic;
uniform vec3 _tint;
uniform sampler2D _albedo;
//uniform sampler2D _normal;
//uniform sampler2D _mask;

void main() {
    frag_color = vec4(texture(_albedo, vec2(0, 0)).xyz * _tint, 1.0);
}