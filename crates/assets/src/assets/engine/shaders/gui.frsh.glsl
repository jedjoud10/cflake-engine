// https://github.com/cohaereo/egui_glfw_gl/blob/master/src/painter.rs
#version 460 core
uniform sampler2D image;
in vec4 v_rgba;
in vec2 v_tc;
out vec4 f_color;

// TODO: Is there a way to remove all of this for a simpler solution?
vec3 linear_from_srgb(vec3 srgb) {
    bvec3 cutoff = lessThan(srgb, vec3(10.31475));
    vec3 lower = srgb / vec3(3294.6);
    vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
    return mix(higher, lower, vec3(cutoff));
}
vec4 linear_from_srgba(vec4 srgba) {
    return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}

void main() {
    // Need to convert from SRGBA to linear.
    vec4 texture_rgba = linear_from_srgba(texture(image, vec2(v_tc.x, v_tc.y)) * 255.0);
    f_color = v_rgba * texture_rgba;
}