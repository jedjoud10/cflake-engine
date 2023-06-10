// Turn the 0-1 depth value to the zNear - zFar range
float linearize_depth(float d,float zNear,float zFar) {
	d = 2.0 * d - 1.0;
    return zNear * zFar / (zFar + d * (zNear - zFar));
}

// Narkowicz 2015, "ACES Filmic Tone Mapping Curve"
vec3 aces(vec3 x) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

// ALU tonemapping
// https://www.shadertoy.com/view/tlfXRB
vec3 alu(vec3 x) {
    vec3 c = max(vec3(0.0), x - 0.004);
    return (c * (c * 6.2 + 0.5)) / (c * (c * 6.2 + 1.7) + 0.06);
}

// randomly found here
// https://www.shadertoy.com/view/Ml2cWG
vec3 reinhard_jodie(vec3 c){
    float l = dot(c, vec3(0.2126, 0.7152, 0.0722));
    vec3 tc = c / (c + 1.0);

    return mix(c / (l + 1.0), tc, tc);
}

// Basic Reinhard tonemapping
// https://64.github.io/tonemapping/
vec3 reinhard(vec3 v) {
    return v / (1.0f + v);
}