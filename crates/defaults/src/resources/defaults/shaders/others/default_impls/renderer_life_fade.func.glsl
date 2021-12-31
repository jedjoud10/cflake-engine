// Some cool dithering at the start of the life of each renderer
ivec2 pixel = ivec2(gl_FragCoord.xy);
if (_fade_anim) {
    if (!_dead) {
        int level = clamp(int(_active_time * 8.0 * 5.0), 0, 5);
        if (get_dither(pixel, level)) { discard; }
    } else {
        int level = clamp(5 - int(_dead_time * 8.0 * 5.0), 0, 5);
        if (get_dither(pixel, level)) { discard; }
    }
}