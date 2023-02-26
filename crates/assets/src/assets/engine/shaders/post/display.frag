#version 460 core
layout(location = 0) out vec4 frag;

// Window bind group buffer (creates a 'window' object)
#include <engine/shaders/common/window.glsl>

void main() {
	// Get the scaled down coordinates
	float x = gl_FragCoord.x / float(window.width);
	float y = gl_FragCoord.y / float(window.height);

	frag = vec4(x, y, 0, 0);
}