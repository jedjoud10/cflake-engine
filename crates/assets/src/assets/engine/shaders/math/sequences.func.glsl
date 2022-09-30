// From https://www.shadertoy.com/view/4dtBWH
// Feturns a 2D quasi-random point using the weyl low discrepency sequence
vec2 weyl(vec2 p0, int n) {
    return fract(p0 + vec2(n*12664745, n*9560333)/exp2(24.));
}


// From https://learnopengl.com/PBR/IBL/Specular-IBL
// returns a 2D quasi-random point using the hammersley low discrepency sequence
vec2 Hammersley(uint i, uint bits)
{
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return float(bits) * 2.3283064365386963e-10; // / 0x100000000
    return vec2(float(i)/float(N), RadicalInverse_VdC(i));
}  