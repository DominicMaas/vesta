#import bevy_pbr::pbr_types;

fn material_from_voxel_index(index: u32) -> pbr_types::StandardMaterial {
    var material = pbr_types::standard_material_new();
    
    material.base_color = vec4<f32>(0.0, 0.0, 1.0, 1.0);
    material.emissive = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    material.perceptual_roughness = 0.8; // 0.5
    material.metallic = 0.00;
    material.reflectance = 0.4; //0.5
    material.diffuse_transmission = 0.0;
    material.specular_transmission = 0.0;
    material.thickness = 0.0;
    material.ior = 1.5;
    material.attenuation_distance = 1.0;
    material.attenuation_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    material.flags = pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE | pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT;
    material.alpha_cutoff = 0.5;
    material.parallax_depth_scale = 0.1;
    material.max_parallax_layer_count = 16.0;
    material.max_relief_mapping_search_steps = 5u;
    material.deferred_lighting_pass_id = 1u;
    
    return material;
}