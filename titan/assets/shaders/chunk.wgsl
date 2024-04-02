// Important: This file is based on the following:
// Vertex: mesh.wgsl
// Fragment: mesh.wgsl

#import bevy_pbr::{
    view_transformations::position_world_to_clip,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
#endif

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

@group(2) @binding(0)
var chunk_texture: texture_2d_array<f32>;

@group(2) @binding(1)
var chunk_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
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
    
    var model = mesh_functions::get_model_matrix(vertex.instance_index);

    out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex.instance_index);

    out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);

    out.uv = vertex.uv;

    out.voxel_index = vertex.voxel_index;
    out.texture_index = vertex.texture_index;

    return out;
}

@fragment
fn fragment(in: MeshVertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
      
    // Construct PBR input
    var pbr_input: pbr_types::PbrInput;
    
    // TODO: Create this material from block
    
    var double_sided = false;  
    let texture = textureSample(chunk_texture, chunk_sampler, in.uv, i32(in.texture_index)); 
    pbr_input.material.base_color = texture;
    
    //  diffuse_occlusion: vec3<f32>,
    
    pbr_input.frag_coord = in.position;
    pbr_input.world_position = in.world_position;
    pbr_input.is_orthographic = view.projection[3].w == 1.0;
    
    pbr_input.V = pbr_functions::calculate_view(in.world_position, pbr_input.is_orthographic);
    
#ifndef LOAD_PREPASS_NORMALS
    pbr_input.N = pbr_functions::apply_normal_mapping(
        pbr_input.material.flags,
        pbr_input.world_normal,
        double_sided,
        is_front,
        in.uv,
        view.mip_bias,
    );
#endif
    
    // lightmap_light: vec3<f32>,
    // flags: u32,
    
   // pbr_input.is_orthographic = view.projection[3].w == 1.0;
   // pbr_input.V = pbr_functions::calculate_view(in.world_position, pbr_input.is_orthographic);
   // 
   // pbr_input.world_position = in.world_position;
    
    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
    
#ifdef PREPASS_PIPELINE
    // write the gbuffer, lighting pass id, and optionally normal and motion_vector textures
    let out = deferred_output(in, pbr_input);
#else
    // in forward mode, we calculate the lit color immediately, and then apply some post-lighting effects here.
    // in deferred mode the lit color and these effects will be calculated in the deferred lighting shader
    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}
    
    /*let is_orthographic = view.projection[3].w == 1.0;
        
   
    
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
}*/