// Load the default uniforms that would normally be used when dealing with renderers
uniform float _time;
uniform float _active_time;
uniform float _dead_time;
uniform ivec2 _resolution;
uniform float _delta;
uniform bool _dead;
const float _FADE_IN_SPEED = #constant fade_in_speed
const float _FADE_OUT_SPEED = #constant fade_out_speed

// Some default functions 
#include "defaults\shaders\others\dithering.func.glsl"