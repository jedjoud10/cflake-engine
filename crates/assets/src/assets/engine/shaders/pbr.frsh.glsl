#version 460 core
out vec4 frag_color;
uniform float _bumpiness;
in vec3 test;
/*
uniform float _roughness;
uniform float _metallic;
uniform sampler2D _albedo;
uniform sampler2D _normal;
uniform sampler2D _mask;
*/


void main() {
    frag_color = vec4(1.0 * _bumpiness * test, 1.0);
}