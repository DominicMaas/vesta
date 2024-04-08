#import bevy_pbr::pbr_types;

const VOXEL_STONE: u32 = 1;
const VOXEL_SAND: u32 = 2;
const VOXEL_GRASS: u32 = 3;
const VOXEL_SNOW: u32 = 4;
const VOXEL_WATER: u32 = 5;

fn material_from_voxel_index(index: u32) -> pbr_types::StandardMaterial {
    var material = pbr_types::standard_material_new();
    material.flags = pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE | pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT;

    
    if (index == VOXEL_STONE) {
        material.base_color = vec4<f32>(0.384, 0.384, 0.384, 1.0);
        material.perceptual_roughness = 0.9; 
        
    } else if (index == VOXEL_SAND) {
        material.base_color = vec4<f32>(0.906, 0.769, 0.588, 1.0);
        material.perceptual_roughness = 0.9;
        
    } else if (index == VOXEL_GRASS) {
        material.base_color = vec4<f32>(0.075, 0.522, 0.063, 1.0);
        material.perceptual_roughness = 0.8;
        
    } else if (index == VOXEL_SNOW) {
        material.base_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        material.perceptual_roughness = 0.5; 
        
    } else if (index == VOXEL_WATER) {
        material.flags = pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT;

        
        material.base_color = vec4<f32>(0.137, 0.537, 0.855, 0.8);
        material.perceptual_roughness = 0.1; 
        material.ior = 1.33;
        material.specular_transmission = 0.5;
        material.thickness = 5.0;
    }
    else
    {
        material.base_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        material.perceptual_roughness = 1.0; 
    }
    
   
    
    material.emissive = vec4<f32>(0.0, 0.0, 0.0, 1.0);
 
    material.metallic = 0.00;
    material.reflectance = 0.4; //0.5
    material.diffuse_transmission = 0.0;
    material.specular_transmission = 0.0;
    
    material.attenuation_distance = 1.0;
    material.attenuation_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    material.alpha_cutoff = 0.5;
    material.parallax_depth_scale = 0.1;
    material.max_parallax_layer_count = 16.0;
    material.max_relief_mapping_search_steps = 5u;
    material.deferred_lighting_pass_id = 1u;
    
    return material;
}