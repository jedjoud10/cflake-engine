// Simple dithering effect with percentage factor
// Percentage of 0 means it's fully dithered, 1 means fully non dithered
bool dither(ivec2 coord, float percentage) {   
    percentage = clamp(percentage, 0, 1); 
    uint index = uint(floor(percentage * 4));
    index = clamp(index, 0, 5);

    if (index == 4) {
        return false;
    }

    bool horizontal = coord.x % (index+1) == 0;
    bool vertical = coord.y % (index+1) == 0;

    return horizontal && vertical;
}

// 3D too cause why not
bool dither(ivec3 coord, float percentage) {   
    percentage = clamp(percentage, 0, 1); 
    uint index = uint(floor(percentage * 4));
    index = clamp(index, 0, 5);

    if (index == 4) {
        return false;
    }

    bool horizontal = coord.x % (index+1) == 0;
    bool vertical = coord.y % (index+1) == 0;
    
    // Note: Lol
    bool depthical = coord.z % (index+1) == 0;

    return horizontal && vertical && depthical;
}