#version 460 core
out vec3 color;
in vec3 m_position;
void main() {
	float val = floor(m_position.x) + floor(m_position.z) + floor(m_position.y);
	val = mod(val, 2.0) == 0 ? 0 : 1;
	color = vec3(val, val, val);
}