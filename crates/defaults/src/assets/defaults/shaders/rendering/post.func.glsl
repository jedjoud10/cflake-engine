// Narkowicz 2015, "ACES Filmic Tone Mapping Curve"
vec3 aces(vec3 x) {
  const float a = 2.51;
  const float b = 0.03;
  const float c = 2.43;
  const float d = 0.59;
  const float e = 0.14;
  return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

float aces(float x) {
  const float a = 2.51;
  const float b = 0.03;
  const float c = 2.43;
  const float d = 0.59;
  const float e = 0.14;
  return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

// Post-rendering effects
vec3 post_rendering(vec2 uvs, vec3 icolor) {
    // A vignette effect
    float vignette_strength_x = pow(abs(uvs.x - 0.5), 4);
    float vignette_strength_y = pow(abs(uvs.y - 0.5), 4);
    float vignette_strength = (vignette_strength_x + vignette_strength_y) * 4.0; 
	return mix(aces(icolor), icolor, 0.2) * (1-vignette_strength);
}