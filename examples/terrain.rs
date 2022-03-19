use std::num::NonZeroU8;

use cflake_engine::{
    assets::assetc,
    defaults::{
        components::{self, Camera, DynamicEdit, Transform},
        globals::{self, TerrainSettings},
    },
    ecs::entity::ComponentLinkingGroup,
    math::octrees::HeuristicSettings,
    rendering::basics::{
        lights::{LightParameters, LightType::Directional},
        material::Material,
        shader::{Shader, ShaderInitSettings},
        texture::{BundledTextureBuilder, Texture2D, TextureFlags, TextureParams},
        uniforms::UniformsSet,
    },
    terrain::editing::{Edit, EditParams},
    vek, World,
};

// A game with some test terrain
fn main() {
    cflake_engine::start("cflake-examples", "terrain", init, cflake_engine::defaults::load_debugging_systems)
}
// Init the terrain world
fn init(world: &mut World) {
    cflake_engine::assets::init!("/examples/assets/");
    cflake_engine::assets::asset!("./assets/user/shaders/voxel_terrain/voxel.func.glsl");
    cflake_engine::assets::asset!("./assets/user/shaders/voxel_terrain/voxel.func.glsl");
    cflake_engine::assets::asset!("./assets/user/textures/Snow006_2K_Color.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/Snow006_2K_NormalGL.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/rocks_ground_06_diff_2k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/rocks_ground_06_nor_gl_2k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/rocks_ground_08_diff_2k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/rocks_ground_08_nor_gl_2k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/snow_01_diff_8k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/snow_01_nor_gl_8k.jpg");
    // Load le assets

    // ----Start the world----
    // Create a simple camera entity
    let mut group = ComponentLinkingGroup::default();
    group.link(Camera::new(90.0, 10.0, 32000.0)).unwrap();
    group.link(Transform::default()).unwrap();
    world.ecs.add(group).unwrap();

    // Create the directional light source
    let light = components::Light {
        light: Directional {
            params: LightParameters {
                strength: 1.0,
                color: vek::Rgb::one(),
            },
        },
    };
    let light_transform = Transform {
        rotation: vek::Quaternion::<f32>::rotation_x(-30f32.to_radians()),
        ..Default::default()
    };
    // And add it to the world as an entity
    let mut group = ComponentLinkingGroup::default();
    group.link(light_transform).unwrap();
    group.link(light).unwrap();
    world.ecs.add(group).unwrap();

    // Load a terrain material
    // Load the shader first
    let settings = ShaderInitSettings::default()
        .source("defaults/shaders/voxel_terrain/terrain.vrsh.glsl")
        .source("defaults/shaders/voxel_terrain/terrain.frsh.glsl");
    let shader = world.pipeline.insert(Shader::new(settings).unwrap());
    // Then the textures
    let texture_diff_1 = assetc::load::<Texture2D>("user/textures/Snow006_2K_Color.jpg").unwrap();
    let texture_norm_1 = assetc::load::<Texture2D>("user/textures/Snow006_2K_NormalGL.jpg").unwrap();
    let texture_diff_2 = assetc::load::<Texture2D>("user/textures/rocks_ground_06_diff_2k.jpg").unwrap();
    let texture_norm_2 = assetc::load::<Texture2D>("user/textures/rocks_ground_06_nor_gl_2k.jpg").unwrap();
    let texture_diff_1 = assetc::load::<Texture2D>("user/textures/forrest_ground_01_diff_2k.jpg").unwrap();
    let texture_norm_1 = assetc::load::<Texture2D>("user/textures/forrest_ground_01_nor_gl_2k.jpg").unwrap();
    let texture_diff_2 = assetc::load::<Texture2D>("user/textures/rocks_ground_06_diff_2k.jpg").unwrap();
    let texture_norm_2 = assetc::load::<Texture2D>("user/textures/rocks_ground_06_nor_gl_2k.jpg").unwrap();
    let texture_diff_3 = assetc::load::<Texture2D>("user/textures/rocks_ground_08_diff_2k.jpg").unwrap();
    let texture_norm_3 = assetc::load::<Texture2D>("user/textures/rocks_ground_08_nor_gl_2k.jpg").unwrap();
    let diffuse = BundledTextureBuilder::build(&[texture_diff_1, texture_diff_2, texture_diff_3], None).unwrap();
    let normals = BundledTextureBuilder::build(
        &[texture_norm_1, texture_norm_2, texture_norm_3],
        Some(TextureParams {
            flags: TextureFlags::MIPMAPS,
            ..Default::default()
        }),
    )
    .unwrap();
    let diffuse = world.pipeline.insert(diffuse);
    let normals = world.pipeline.insert(normals);
    let material = Material {
        shader,
        uniforms: UniformsSet::new(move |mut uniforms| {
            // Set the textures first
            uniforms.set_bundled_texture2d("diffuse_m", &diffuse);
            uniforms.set_bundled_texture2d("normal_m", &normals);
            // Then the parameters
            uniforms.set_f32("bumpiness", 3.0);
            uniforms.set_vec2f32("uv_scale", vek::Vec2::broadcast(0.02));
        }),
    };
    let material = world.pipeline.insert(material);
    let heuristic = HeuristicSettings {
        function: |node, target| {
            let dist = vek::Vec3::<f32>::distance(node.center().as_(), *target) / (node.half_extent().get() as f32 * 2.0);
            dist < 1.2
        },
    };
    // Create some terrain settings
    let terrain_settings = TerrainSettings {
        voxel_src_path: "user/shaders/voxel_terrain/voxel.func.glsl".to_string(),
        depth: NonZeroU8::new(4).unwrap(),
        heuristic_settings: heuristic,
        material,
        physics: false,
        ..Default::default()
    };
    let mut terrain = globals::Terrain::new(&world.settings.terrain, terrain_settings, &mut world.pipeline);
    // Big sphere
    terrain.edit(Edit::sphere(
        vek::Vec3::unit_y() * -50.0,
        50.0,
        EditParams {
            _union: true,
            material: None,
            ..Default::default()
        },
    ));
    world.globals.add(terrain).unwrap();

    // And add it to the world as an entity
    let mut group = ComponentLinkingGroup::default();
    group.link(Transform::default()).unwrap();
    group.link(DynamicEdit::new(Edit::sphere(vek::Vec3::default(), 200.0, EditParams::default()))).unwrap();
    world.ecs.add(group).unwrap();
}
