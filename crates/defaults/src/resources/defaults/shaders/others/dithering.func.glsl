// Some dithering patterns in 1d
bool pattern[6][5] = bool[][](bool[](true, true, true, true, true),
    bool[](true, true, false, true, true),
    bool[](true, false, true, false, true),
    bool[](false, true, false, true, false),
    bool[](false, false, true, false, false),
    bool[](false, false, false, false, false)); 

// Get a dither at a specific level
bool get_dither(ivec2 pixel, int level) {
    level = level % 6;
    return pattern[level][pixel.x % 5] && pattern[level][(pixel.y + 1) % 5];
}