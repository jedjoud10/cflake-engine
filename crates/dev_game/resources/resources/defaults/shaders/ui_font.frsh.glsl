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
    float alpha = signed_distance > 0.2 ? 1 : 0;

    // Outline
    vec3 fcolor = vec3(signed_distance, signed_distance, signed_distance);    
    if (signed_distance > 0.4) {
        fcolor = vec3(0, 0, 0);
    } else {
        fcolor = vec3(1, 1, 1);
    }        
    
	out_color = vec4(fcolor, alpha);
}