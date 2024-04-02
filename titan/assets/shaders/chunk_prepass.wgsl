
#import bevy_pbr::pbr_functions
#import bevy_pbr::pbr_types as pbr_types

#import bevy_pbr::mesh_bindings             mesh

#import bevy_pbr::{
    pbr_prepass_functions,
    prepass_bindings,
    prepass_io::{VertexOutput, FragmentOutput},
    mesh_functions,
    skinning,
    morph,
    mesh_view_bindings::{view, previous_view_proj},
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) voxel_index: u32,
    @location(4) texture_index: u32,
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
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    var model = mesh_functions::get_model_matrix(vertex_no_morph.instance_index);
#endif // SKINNED
    out.position = mesh_functions::mesh_position_local_to_clip(model, vec4(vertex.position, 1.0));
#ifdef DEPTH_CLAMP_ORTHO
    out.clip_position_unclamped = out.position;
    out.position.z = min(out.position.z, 1.0);
#endif // DEPTH_CLAMP_ORTHO

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif // VERTEX_UVS

#ifdef NORMAL_PREPASS
#ifdef SKINNED
    out.world_normal = bevy_pbr::skinning::skin_normals(model, vertex.normal);
#else // SKINNED
    out.world_normal = bevy_pbr::mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif // SKINNED

#ifdef VERTEX_TANGENTS
    out.world_tangent = bevy_pbr::mesh_functions::mesh_tangent_local_to_world(
        model,
        vertex.tangent,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif // VERTEX_TANGENTS
#endif // NORMAL_PREPASS

#ifdef MOTION_VECTOR_PREPASS
    out.world_position = bevy_pbr::mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    out.previous_world_position = bevy_pbr::mesh_functions::mesh_position_local_to_world(
        mesh[vertex_no_morph.instance_index].previous_model,
        vec4<f32>(vertex.position, 1.0)
    );
#endif // MOTION_VECTOR_PREPASS

    return out;
}

/// PBR FRAGEMENT

#ifdef PREPASS_FRAGMENT
@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    pbr_prepass_functions::prepass_alpha_discard(in);
    
    var out: FragmentOutput;
    
    #ifdef DEPTH_CLAMP_ORTHO
        out.frag_depth = in.clip_position_unclamped.z;
    #endif // DEPTH_CLAMP_ORTHO
    
    // TODO: Create this material from block
    var material = pbr_types::standard_material_new();
    material.perceptual_roughness = 0.0;
    material.reflectance = 0.0;
    material.metallic = 0.0;
    material.emissive = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    material.flags |= pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT;
        
    #ifdef NORMAL_PREPASS
    // NOTE: Unlit bit not set means == 0 is true, so the true case is if lit
    if (material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        let double_sided = (material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

        let world_normal = pbr_functions::prepare_world_normal(
            in.world_normal,
            double_sided,
            is_front,
        );

        let normal = pbr_functions::apply_normal_mapping(
            material.flags,
            world_normal,
            double_sided,
            is_front,
#ifdef VERTEX_TANGENTS
#ifdef STANDARD_MATERIAL_NORMAL_MAP
            in.world_tangent,
#endif // STANDARD_MATERIAL_NORMAL_MAP
#endif // VERTEX_TANGENTS
#ifdef VERTEX_UVS
            in.uv,
#endif // VERTEX_UVS
            view.mip_bias,
        );

        out.normal = vec4(normal * 0.5 + vec3(0.5), 1.0);
    } else {
        out.normal = vec4(in.world_normal * 0.5 + vec3(0.5), 1.0);
    }
#endif // NORMAL_PREPASS

#ifdef MOTION_VECTOR_PREPASS
    out.motion_vector = pbr_prepass_functions::calculate_motion_vector(in.world_position, in.previous_world_position);
#endif

    return out;
}
#else
@fragment
fn fragment(in: VertexOutput) {
    pbr_prepass_functions::prepass_alpha_discard(in);
}
#endif // PREPASS_FRAGMENT