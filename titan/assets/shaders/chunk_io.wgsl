struct ChunkVertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) voxel_index: u32,
}

#ifdef PREPASS_PIPELINE
struct ChunkVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) previous_world_position: vec4<f32>,  
    @location(2) world_normal: vec3<f32>, 
    @location(3) clip_position_unclamped: vec4<f32>,
    @location(4) voxel_index: u32,
    @location(5) instance_index: u32,   
}
#else
struct ChunkVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>, 
    @location(2) voxel_index: u32,
    @location(3) @interpolate(flat) instance_index: u32,
}
#endif