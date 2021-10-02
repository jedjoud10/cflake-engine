#version 460 core
out vec4 out_color;
in vec2 uvs;
uniform sampler2D atlas_texture;
uniform float time;
uniform vec4 color;
void main() {    
    // Get the color of this specific pixel in the font atlas
    float signed_distance = texture(atlas_texture, uvs).x;    
    // Detect alpha clipping
    float alpha = signed_distance < 0.1 ? 0 : 1;

    // Outline
    if (signed_distance > 0.3) {
        signed_distance = 0;
    } else {
        signed_distance = 1;
    }    
    
	out_color = vec4(signed_distance, signed_distance, signed_distance, 1);
}