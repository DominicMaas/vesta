// Important: This file is based on the following:
// Vertex: mesh.wgsl
// Fragment: mesh.wgsl

#import bevy_core_pipeline::tonemapping     screen_space_dither, powsafe, tone_mapping

#import bevy_pbr::mesh_functions as mesh_functions
#import bevy_pbr::mesh_bindings             mesh
#import bevy_pbr::mesh_view_bindings        view, fog, screen_space_ambient_occlusion_texture
#import bevy_pbr::mesh_view_types           FOG_MODE_OFF

#import bevy_pbr::pbr_functions as pbr_functions
#import bevy_pbr::pbr_types as pbr_types

#import bevy_pbr::prepass_utils

#ifdef SCREEN_SPACE_AMBIENT_OCCLUSION
#import bevy_pbr::gtao_utils gtao_multibounce
#endif

@group(1) @binding(0)
var chunk_texture: texture_2d_array<f32>;

@group(1) @binding(1)
var chunk_sampler: sampler;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) voxel_index: u32,
    @location(4) texture_index: u32,
};

struct MeshVertexOutput {
    // this is `clip position` when the struct is used as a vertex stage output 
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    
    // Position in the world
    @location(0) world_position: vec4<f32>,
    
    // Normals for lighting
    @location(1) world_normal: vec3<f32>,
    
    // UV coords for textures
    @location(2) uv: vec2<f32>,

    // Used to lookup our voxel index in the material array
    @location(3) voxel_index: u32,
    
    // Used to lookup what texture we should use for the face
    @location(4) texture_index: u32,
};

@vertex
fn vertex(vertex: Vertex) -> MeshVertexOutput {
    var out: MeshVertexOutput;

    out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal);

    out.world_position = mesh_functions::mesh_position_local_to_world(mesh.model, vec4<f32>(vertex.position, 1.0));
    out.position = mesh_functions::mesh_position_world_to_clip(out.world_position);

    out.uv = vertex.uv;

    out.voxel_index = vertex.voxel_index;
    out.texture_index = vertex.texture_index;

    return out;
}

@fragment
fn fragment(in: MeshVertexOutput, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    
    // TODO: Create this material from block
    var material = pbr_types::standard_material_new();
    material.perceptual_roughness = 0.0;
    material.reflectance = 0.0;
    material.metallic = 0.0;
    material.emissive = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    
    var pbr_input: pbr_functions::PbrInput;
    pbr_input.material = material;
    
    let is_orthographic = view.projection[3].w == 1.0;
        
    let texture = textureSample(chunk_texture, chunk_sampler, in.uv, i32(in.texture_index)); 
    pbr_input.material.base_color = texture;
    
    var occlusion: vec3<f32> = vec3(1.0);
    
#ifdef SCREEN_SPACE_AMBIENT_OCCLUSION
    let ssao = textureLoad(screen_space_ambient_occlusion_texture, vec2<i32>(in.position.xy), 0i).r;
    let ssao_multibounce = gtao_multibounce(ssao, pbr_input.material.base_color.rgb);
    occlusion = min(occlusion, ssao_multibounce);
#endif
    
    pbr_input.occlusion = occlusion;
    
    pbr_input.frag_coord = in.position;
    pbr_input.world_position = in.world_position;
    
    pbr_input.world_normal = pbr_functions::prepare_world_normal(
        in.world_normal,
        (pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
        is_front,
    );
    
    pbr_input.is_orthographic = is_orthographic;
    
#ifdef LOAD_PREPASS_NORMALS
    pbr_input.N = bevy_pbr::prepass_utils::prepass_normal(in.position, 0u);
#else
    pbr_input.N = pbr_functions::apply_normal_mapping(
        pbr_input.material.flags,
        pbr_input.world_normal,
        in.uv,
        view.mip_bias,
    );
#endif

    pbr_input.V = pbr_functions::calculate_view(in.world_position, is_orthographic);

    // Calculate PBR Values
    var output_color = pbr_functions::pbr(pbr_input);

    // fog
    if (fog.mode != FOG_MODE_OFF && (pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT) != 0u) {
        output_color = pbr_functions::apply_fog(fog, output_color, in.world_position.xyz, view.world_position.xyz);
    }

#ifdef TONEMAP_IN_SHADER
    output_color = tone_mapping(output_color, view.color_grading);
#ifdef DEBAND_DITHER
    var output_rgb = output_color.rgb;
    output_rgb = powsafe(output_rgb, 1.0 / 2.2);
    output_rgb = output_rgb + screen_space_dither(in.position.xy);
    // This conversion back to linear space is required because our output texture format is
    // SRGB; the GPU will assume our output is linear and will apply an SRGB conversion.
    output_rgb = powsafe(output_rgb, 2.2);
    output_color = vec4(output_rgb, output_color.a);
#endif
#endif
#ifdef PREMULTIPLY_ALPHA
    output_color = pbr_functions::premultiply_alpha(pbr_input.material.flags, output_color);
#endif
    return output_color;
}