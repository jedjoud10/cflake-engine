#version 460 core
out vec4 out_color;
in vec2 uvs;
uniform sampler2D atlas_texture;
uniform float time;
uniform vec4 color;
// Font options
uniform vec4 font_color;
uniform vec4 font_outline_color;
uniform float font_thickness;
uniform float font_outline_thickness;

void main() {    
    // Get the color of this specific pixel in the font atlas
    float signed_distance = texture(atlas_texture, uvs).x;    
    vec3 fcolor = font_color.xyz;    
    float alpha = smoothstep(0, font_thickness, signed_distance);
    float k = smoothstep(font_thickness, font_outline_thickness, signed_distance);
    fcolor = mix(fcolor, font_outline_color.xyz, 1-k);
	out_color = vec4(fcolor, alpha);
}