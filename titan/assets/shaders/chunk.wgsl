#import bevy_pbr::{
    mesh_functions,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types,
    pbr_fragment,
    view_transformations::position_world_to_clip,
    forward_io::{FragmentOutput, VertexOutput},
}

#import "shaders/utils/custom_material_functions.wgsl"::titan_pbr_input_from_standard_material
#import "shaders/chunk_io.wgsl"::{ChunkVertex, ChunkVertexOutput}
#import "shaders/utils/chunk_material.wgsl"::material_from_voxel_index

@vertex
fn vertex(vertex: ChunkVertex) -> ChunkVertexOutput {
    var out: ChunkVertexOutput;
 
    var model = mesh_functions::get_model_matrix(vertex.instance_index);
    
    out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
    
    out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
    
    out.instance_index = vertex.instance_index;
    out.voxel_index = vertex.voxel_index;

    return out;
}

@fragment
fn fragment(in: ChunkVertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var out: FragmentOutput;
    
    let material = material_from_voxel_index(in.voxel_index);
    
    var bevyVO: VertexOutput;
    bevyVO.position = in.position;
    bevyVO.world_position = in.world_position;
    bevyVO.world_normal = in.world_normal;
    bevyVO.instance_index = in.instance_index;
    
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = titan_pbr_input_from_standard_material(bevyVO, is_front, material);

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    if (pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }

    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}