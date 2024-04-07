#import bevy_pbr::{
    mesh_functions,
    pbr_types,
    pbr_functions,
    pbr_prepass_functions,
    mesh_view_bindings::view,
    prepass_io::FragmentOutput
}

#import "shaders/chunk_io.wgsl"::{ChunkVertex, ChunkVertexOutput}
#import "shaders/utils/chunk_material.wgsl"::material_from_voxel_index

// Cutoff used for the premultiplied alpha modes BLEND and ADD.
const PREMULTIPLIED_ALPHA_CUTOFF = 0.05;

// We can use a simplified version of alpha_discard() here since we only need to handle the alpha_cutoff
// Based on bevy_pbr_preepass_functions but with taking in a material
fn titan_prepass_alpha_discard(material: pbr_types::StandardMaterial) {

    var output_color: vec4<f32> = material.base_color;

    let alpha_mode = material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
    if alpha_mode == pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_MASK {
        if output_color.a < material.alpha_cutoff {
            discard;
        }
    } else if (alpha_mode == pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND || alpha_mode == pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_ADD) {
        if output_color.a < PREMULTIPLIED_ALPHA_CUTOFF {
            discard;
        }
    } else if alpha_mode == pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_PREMULTIPLIED {
        if all(output_color < vec4(PREMULTIPLIED_ALPHA_CUTOFF)) {
            discard;
        }
    }
}

@vertex
fn vertex(vertex: ChunkVertex) -> ChunkVertexOutput {
    var out: ChunkVertexOutput;
    
    var model = mesh_functions::get_model_matrix(vertex.instance_index);
    out.position = mesh_functions::mesh_position_local_to_clip(model, vec4(vertex.position, 1.0));
    
#ifdef DEPTH_CLAMP_ORTHO
    out.clip_position_unclamped = out.position;
    out.position.z = min(out.position.z, 1.0);
#endif // DEPTH_CLAMP_ORTHO

#ifdef NORMAL_PREPASS_OR_DEFERRED_PREPASS
    out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
#endif // NORMAL_PREPASS_OR_DEFERRED_PREPASS
    
    out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    
#ifdef MOTION_VECTOR_PREPASS
    out.previous_world_position = mesh_functions::mesh_position_local_to_world(
        mesh_functions::get_previous_model_matrix(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0)
    );
#endif // MOTION_VECTOR_PREPASS

    out.instance_index = vertex.instance_index;
    out.voxel_index = vertex.voxel_index;

    return out;
}

@fragment
fn fragment(
    in: ChunkVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    let material = material_from_voxel_index(in.voxel_index);
    
    titan_prepass_alpha_discard(material);

    var out: FragmentOutput;
    
#ifdef DEPTH_CLAMP_ORTHO
    out.frag_depth = in.clip_position_unclamped.z;
#endif // DEPTH_CLAMP_ORTHO
    
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