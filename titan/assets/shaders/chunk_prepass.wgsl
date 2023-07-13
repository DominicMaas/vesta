#import bevy_pbr::mesh_functions

#import bevy_pbr::pbr_functions
#import bevy_pbr::pbr_types as pbr_types

#import bevy_pbr::prepass_bindings
#import bevy_pbr::mesh_bindings             mesh

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) voxel_index: u32,
    @location(4) texture_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,

#ifdef VERTEX_UVS
    @location(0) uv: vec2<f32>,
#endif // VERTEX_UVS

#ifdef NORMAL_PREPASS
    @location(1) world_normal: vec3<f32>,
#ifdef VERTEX_TANGENTS
    @location(2) world_tangent: vec4<f32>,
#endif // VERTEX_TANGENTS
#endif // NORMAL_PREPASS

#ifdef MOTION_VECTOR_PREPASS
    @location(3) world_position: vec4<f32>,
    @location(4) previous_world_position: vec4<f32>,
#endif // MOTION_VECTOR_PREPASS

#ifdef DEPTH_CLAMP_ORTHO
    @location(5) clip_position_unclamped: vec4<f32>,
#endif // DEPTH_CLAMP_ORTHO
}

#ifdef MORPH_TARGETS
fn morph_vertex(vertex_in: Vertex) -> Vertex {
    var vertex = vertex_in;
    let weight_count = bevy_pbr::morph::layer_count();
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = bevy_pbr::morph::weight_at(i);
        if weight == 0.0 {
            continue;
        }
        vertex.position += weight * bevy_pbr::morph::morph(vertex.index, bevy_pbr::morph::position_offset, i);
#ifdef VERTEX_NORMALS
        vertex.normal += weight * bevy_pbr::morph::morph(vertex.index, bevy_pbr::morph::normal_offset, i);
#endif
#ifdef VERTEX_TANGENTS
        vertex.tangent += vec4(weight * bevy_pbr::morph::morph(vertex.index, bevy_pbr::morph::tangent_offset, i), 0.0);
#endif
    }
    return vertex;
}
#endif

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph);
#else
    var vertex = vertex_no_morph;
#endif

#ifdef SKINNED
    var model = bevy_pbr::skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
#else // SKINNED
    var model = mesh.model;
#endif // SKINNED

    out.clip_position = bevy_pbr::mesh_functions::mesh_position_local_to_clip(model, vec4(vertex.position, 1.0));
#ifdef DEPTH_CLAMP_ORTHO
    out.clip_position_unclamped = out.clip_position;
    out.clip_position.z = min(out.clip_position.z, 1.0);
#endif // DEPTH_CLAMP_ORTHO

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif // VERTEX_UVS

#ifdef NORMAL_PREPASS
#ifdef SKINNED
    out.world_normal = bevy_pbr::skinning::skin_normals(model, vertex.normal);
#else // SKINNED
    out.world_normal = bevy_pbr::mesh_functions::mesh_normal_local_to_world(vertex.normal);
#endif // SKINNED

#ifdef VERTEX_TANGENTS
    out.world_tangent = bevy_pbr::mesh_functions::mesh_tangent_local_to_world(model, vertex.tangent);
#endif // VERTEX_TANGENTS
#endif // NORMAL_PREPASS

#ifdef MOTION_VECTOR_PREPASS
    out.world_position = bevy_pbr::mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.previous_world_position = bevy_pbr::mesh_functions::mesh_position_local_to_world(mesh.previous_model, vec4<f32>(vertex.position, 1.0));
#endif // MOTION_VECTOR_PREPASS

    return out;
}

/// PBR FRAGEMENT

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
#ifdef VERTEX_UVS
    @location(0) uv: vec2<f32>,
#endif // VERTEX_UVS

#ifdef NORMAL_PREPASS
    @location(1) world_normal: vec3<f32>,
#ifdef VERTEX_TANGENTS
    @location(2) world_tangent: vec4<f32>,
#endif // VERTEX_TANGENTS
#endif // NORMAL_PREPASS

#ifdef MOTION_VECTOR_PREPASS
    @location(3) world_position: vec4<f32>,
    @location(4) previous_world_position: vec4<f32>,
#endif // MOTION_VECTOR_PREPASS

#ifdef DEPTH_CLAMP_ORTHO
    @location(5) clip_position_unclamped: vec4<f32>,
#endif // DEPTH_CLAMP_ORTHO
};

// Cutoff used for the premultiplied alpha modes BLEND and ADD.
const PREMULTIPLIED_ALPHA_CUTOFF = 0.05;

// We can use a simplified version of alpha_discard() here since we only need to handle the alpha_cutoff
fn prepass_alpha_discard(in: FragmentInput) {

#ifdef MAY_DISCARD
    var output_color: vec4<f32> = bevy_pbr::pbr_bindings::material.base_color;

#ifdef VERTEX_UVS
    if (bevy_pbr::pbr_bindings::material.flags & bevy_pbr::pbr_types::STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0u {
        output_color = output_color * textureSampleBias(bevy_pbr::pbr_bindings::base_color_texture, bevy_pbr::pbr_bindings::base_color_sampler, in.uv, bevy_pbr::prepass_bindings::view.mip_bias);
    }
#endif // VERTEX_UVS

    let alpha_mode = bevy_pbr::pbr_bindings::material.flags & bevy_pbr::pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
    if alpha_mode == bevy_pbr::pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_MASK {
        if output_color.a < bevy_pbr::pbr_bindings::material.alpha_cutoff {
            discard;
        }
    } else if (alpha_mode == bevy_pbr::pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND || alpha_mode == bevy_pbr::pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_ADD) {
        if output_color.a < PREMULTIPLIED_ALPHA_CUTOFF {
            discard;
        }
    } else if alpha_mode == bevy_pbr::pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_PREMULTIPLIED {
        if all(output_color < vec4(PREMULTIPLIED_ALPHA_CUTOFF)) {
            discard;
        }
    }

#endif // MAY_DISCARD
}

#ifdef PREPASS_FRAGMENT
struct FragmentOutput {
#ifdef NORMAL_PREPASS
    @location(0) normal: vec4<f32>,
#endif // NORMAL_PREPASS

#ifdef MOTION_VECTOR_PREPASS
    @location(1) motion_vector: vec2<f32>,
#endif // MOTION_VECTOR_PREPASS

#ifdef DEPTH_CLAMP_ORTHO
    @builtin(frag_depth) frag_depth: f32,
#endif // DEPTH_CLAMP_ORTHO
}

@fragment
fn fragment(in: FragmentInput) -> FragmentOutput {
    
    // TODO: Create this material from block
    var material = pbr_types::standard_material_new();
    material.perceptual_roughness = 0.0;
    material.reflectance = 0.0;
    material.metallic = 0.0;
    material.emissive = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    material.flags |= pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT;
        
    prepass_alpha_discard(in);

    var out: FragmentOutput;

#ifdef DEPTH_CLAMP_ORTHO
    out.frag_depth = in.clip_position_unclamped.z;
#endif // DEPTH_CLAMP_ORTHO

#ifdef NORMAL_PREPASS
    // NOTE: Unlit bit not set means == 0 is true, so the true case is if lit
    if (material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        let world_normal = bevy_pbr::pbr_functions::prepare_world_normal(
            in.world_normal,
            (material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
            in.is_front,
        );

        let normal = bevy_pbr::pbr_functions::apply_normal_mapping(
            material.flags,
            world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
            in.world_tangent,
#endif // STANDARDMATERIAL_NORMAL_MAP
#endif // VERTEX_TANGENTS
#ifdef VERTEX_UVS
            in.uv,
#endif // VERTEX_UVS
            bevy_pbr::prepass_bindings::view.mip_bias,
        );

        out.normal = vec4(normal * 0.5 + vec3(0.5), 1.0);
    } else {
        out.normal = vec4(in.world_normal * 0.5 + vec3(0.5), 1.0);
    }
#endif // NORMAL_PREPASS

    return out;
}
#else
@fragment
fn fragment(in: FragmentInput) {
    prepass_alpha_discard(in);
}
#endif // PREPASS_FRAGMENT