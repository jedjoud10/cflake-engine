// A packed terrain edit that influences the terrain
struct PackedTerrainEdit {
    // XYZ position and RGB color
    uint x_y;
    uint z_sx;
    uint sy_sz;

    // 2 bytes for color, 4 bits for shape type, 4 bits for edit type, 1 byte for material
    uint rgbcolor_shapetype_edittype_material;
};

// The unpacked version of a terrain edit
struct TerrainEdit {
    vec3 position;
    vec3 color;
    uint shapetype;
    uint editype;
    uint material;
};

// Unpack a packed terrain edit
TerrainEdit get_unpacked_terrain_edit(PackedTerrainEdit packed) {

}