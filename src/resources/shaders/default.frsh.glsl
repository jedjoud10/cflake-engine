#version 330 core
out vec3 color;
uniform float test;

void main(){
	vec3 position = gl_FragCoord.xyz;
  	color = position;
}