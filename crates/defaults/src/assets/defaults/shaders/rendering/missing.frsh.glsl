#version 460 core

// Pixel color
out vec4 frag_color;

void main() {
	// For the missing shader, just return a debug value
	frag_color = vec4(1, 0, 1, 0);
}