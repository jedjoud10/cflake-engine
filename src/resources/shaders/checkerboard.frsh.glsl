#version 460 core
out vec3 color;
uniform vec3 rgb;
in vec3 world_position;
void main() {
	float val = floor(world_position.x) + floor(world_position.z) + floor(world_position.y);
	val = mod(val, 2.0) == 0 ? 0 : 1;
	color = vec3(val, val, val);
}