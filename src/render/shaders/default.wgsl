#import bevy_terrain::types

struct TerrainConfig {
    lod_count: u32,
    height: f32,
    chunk_size: u32,
    terrain_size: u32,

    height_scale: f32,
    density_scale: f32,
    _empty: u32,
    _empty: u32,
    height_offset: f32,
    density_offset: f32,
    _empty: u32,
    _empty: u32,
}

// view bindings
#import bevy_pbr::mesh_view_bindings

// terrain view bindings
@group(1) @binding(0)
var<uniform> view_config: TerrainViewConfig;
@group(1) @binding(1)
var quadtree: texture_2d_array<u32>;
@group(1) @binding(2)
var<storage> tiles: TileList;

// terrain bindings
@group(2) @binding(0)
var<uniform> config: TerrainConfig;
@group(2) @binding(1)
var terrain_sampler: sampler;
@group(2) @binding(2)
var height_atlas: texture_2d_array<f32>;
@group(2) @binding(3)
var density_atlas: texture_2d_array<f32>;

#import bevy_pbr::mesh_types
#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

#import bevy_terrain::atlas
#import bevy_terrain::functions
#import bevy_terrain::debug

fn color_fragment(
    in: FragmentInput,
    lod: u32,
    atlas_index: i32,
    atlas_coords: vec2<f32>
) -> vec4<f32> {
    var color = vec4<f32>(0.5);

    let height_coords = atlas_coords * config.height_scale + config.height_offset;
    let world_normal = calculate_normal(height_coords, atlas_index, lod);

    #ifdef SHOW_LOD
        color = mix(color, show_lod(lod, in.world_position.xyz), 0.4);
    #endif

    #ifdef SHOW_UV
        color = mix(color, vec4<f32>(atlas_coords.x, atlas_coords.y, 0.0, 1.0), 0.5);
    #endif

    #ifdef LIGHTING
        var pbr_input: PbrInput = pbr_input_new();
        pbr_input.material.base_color = color;
        pbr_input.material.perceptual_roughness = 0.6;
        pbr_input.material.reflectance = 0.1;
        pbr_input.frag_coord = in.frag_coord;
        pbr_input.world_position = in.world_position;
        pbr_input.world_normal = world_normal;
        pbr_input.is_orthographic = view.projection[3].w == 1.0;
        pbr_input.N = world_normal;
        pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);

        color = pbr(pbr_input);
    #endif

    return color;
}

#import bevy_terrain::entry_points