#include "defaults/shaders/others/sdf.func.glsl"
// A packed terrain edit that influences the terrain
struct PackedTerrainEdit {
    // XYZ position and RGB color
    uint x_y;
    uint z_sx;
    uint sy_sz;
    // 2 bytes for color, 4 bits for shape type, 4 bits for edit type, 1 byte for material
    uint rgbcolor_shape_type_edit_type_material;
};

// The unpacked version of a terrain edit
struct TerrainEdit {
    vec3 position;
    vec3 size;
    vec3 color;
    uint shape_type;
    uint edit_type;
    uint material;
};

// Unpack a packed terrain edit
TerrainEdit get_unpacked_terrain_edit(PackedTerrainEdit edit) {
    // Decode position and size
    vec3 pos = vec3(0);
    pos.xy = unpackHalf2x16(edit.x_y);
    vec2 z_sx = unpackHalf2x16(edit.z_sx);
    pos.z = z_sx.x;
    vec2 sy_sz = unpackHalf2x16(edit.sy_sz);
    vec3 size = vec3(z_sx.y, sy_sz);
    // Decode color, shape_type, edit_type, and material
    uint packed_color = edit.rgbcolor_shape_type_edit_type_material >> 16;
    vec3 color = vec3(0);
    color.r = clamp(float((packed_color >> 11) * 8), 0, 255);
    color.g = clamp(float(((packed_color >> 5) & 63) * 4), 0, 255);
    color.b = clamp(float(((packed_color) & 31) * 8), 0, 255);
    color /= 255.0;
    // Decode shape_type, edit_type, and material
    uint shape_type_edit_type_material = edit.rgbcolor_shape_type_edit_type_material & 65535;
    uint shape_type_edit_type = shape_type_edit_type_material >> 8;
    uint shape_type = (shape_type_edit_type >> 4) & 15;
    uint material = shape_type_edit_type_material & 255;
    uint edit_type = shape_type_edit_type & 15;
    return TerrainEdit(pos, size, color, shape_type, edit_type, material);
}


// Update a density function using a single edit
void edit_density(const vec3 pos, inout float density, inout vec3 color, inout uint material, TerrainEdit edit) {
    // Get the shape density first
    float shape_density = 0.0;
    const float threshold = 5.0;
    if (edit.shape_type == 0) {
        shape_density = sdBox(pos-edit.position, edit.size / 2.0);
    } else if (edit.shape_type == 1) {
        shape_density = sdSphere(pos-edit.position, edit.size.x);    
    }
    // Then combine it
    if (edit.edit_type == 0) {
        density = opUnion(density, shape_density);
    } else if (edit.edit_type == 1) {
        density = opSubtraction(shape_density, density);    
    }
    // Color
    if (shape_density < threshold) {
        color = edit.color;
    }
    // Material
    if (edit.material != 255 && shape_density < threshold) {
        material = 0;
    }
}