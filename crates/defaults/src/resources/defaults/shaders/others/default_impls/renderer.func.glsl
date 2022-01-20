// Load the default uniforms that would normally be used when dealing with renderers
uniform float _active_time;
uniform float _dead_time;
uniform bool _dead;
uniform bool _fade_anim;
const float _FADE_IN_SPEED = #constant fade_in_speed
const float _FADE_OUT_SPEED = #constant fade_out_speed

// Some default functions 
#include "defaults\shaders\others\dithering.func.glsl"