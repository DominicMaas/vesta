#import bevy_pbr::{
    pbr_functions,
    pbr_bindings,
    pbr_fragment,
    pbr_types,
    prepass_utils,
    lighting,
    mesh_bindings::mesh,
    mesh_view_bindings::view,
    parallax_mapping::parallaxed_uv,
    lightmap::lightmap,
    forward_io::{VertexOutput, FragmentOutput},
}

#ifdef SCREEN_SPACE_AMBIENT_OCCLUSION
#import bevy_pbr::mesh_view_bindings::screen_space_ambient_occlusion_texture
#import bevy_pbr::gtao_utils::gtao_multibounce
#endif

// Prepare a full PbrInput by sampling all textures to resolve
// the material members
fn titan_pbr_input_from_standard_material(
    in: VertexOutput,
    is_front: bool,
    material: pbr_types::StandardMaterial,
) -> pbr_types::PbrInput {
    let double_sided = (material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

    var pbr_input: pbr_types::PbrInput = pbr_fragment::pbr_input_from_vertex_output(in, is_front, double_sided);
    pbr_input.material.flags = material.flags;
    pbr_input.material.base_color *= material.base_color;
    pbr_input.material.deferred_lighting_pass_id = material.deferred_lighting_pass_id;

    // Neubelt and Pettineo 2013, "Crafting a Next-gen Material Pipeline for The Order: 1886"
    let NdotV = max(dot(pbr_input.N, pbr_input.V), 0.0001);

#ifdef VERTEX_UVS
    var uv = in.uv;

#ifdef VERTEX_TANGENTS
    if ((material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DEPTH_MAP_BIT) != 0u) {
        let V = pbr_input.V;
        let N = in.world_normal;
        let T = in.world_tangent.xyz;
        let B = in.world_tangent.w * cross(N, T);
        // Transform V from fragment to camera in world space to tangent space.
        let Vt = vec3(dot(V, T), dot(V, B), dot(V, N));
        uv = parallaxed_uv(
            material.parallax_depth_scale,
            material.max_parallax_layer_count,
            material.max_relief_mapping_search_steps,
            uv,
            // Flip the direction of Vt to go toward the surface to make the
            // parallax mapping algorithm easier to understand and reason
            // about.
            -Vt,
        );
    }
#endif // VERTEX_TANGENTS

    if ((material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0u) {
        pbr_input.material.base_color *= textureSampleBias(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv, view.mip_bias);
    }
#endif // VERTEX_UVS

    pbr_input.material.flags = material.flags;

    // NOTE: Unlit bit not set means == 0 is true, so the true case is if lit
    if ((material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u) {
        pbr_input.material.reflectance = material.reflectance;
        pbr_input.material.ior = material.ior;
        pbr_input.material.attenuation_color = material.attenuation_color;
        pbr_input.material.attenuation_distance = material.attenuation_distance;
        pbr_input.material.alpha_cutoff = material.alpha_cutoff;

        // emissive
        // TODO use .a for exposure compensation in HDR
        var emissive: vec4<f32> = material.emissive;
#ifdef VERTEX_UVS
        if ((material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_EMISSIVE_TEXTURE_BIT) != 0u) {
            emissive = vec4<f32>(emissive.rgb * textureSampleBias(pbr_bindings::emissive_texture, pbr_bindings::emissive_sampler, uv, view.mip_bias).rgb, 1.0);
        }
#endif
        pbr_input.material.emissive = emissive;

        // metallic and perceptual roughness
        var metallic: f32 = material.metallic;
        var perceptual_roughness: f32 = material.perceptual_roughness;
        let roughness = lighting::perceptualRoughnessToRoughness(perceptual_roughness);
#ifdef VERTEX_UVS
        if ((material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BIT) != 0u) {
            let metallic_roughness = textureSampleBias(pbr_bindings::metallic_roughness_texture, pbr_bindings::metallic_roughness_sampler, uv, view.mip_bias);
            // Sampling from GLTF standard channels for now
            metallic *= metallic_roughness.b;
            perceptual_roughness *= metallic_roughness.g;
        }
#endif
        pbr_input.material.metallic = metallic;
        pbr_input.material.perceptual_roughness = perceptual_roughness;

        var specular_transmission: f32 = material.specular_transmission;
#ifdef PBR_TRANSMISSION_TEXTURES_SUPPORTED
        if ((material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_SPECULAR_TRANSMISSION_TEXTURE_BIT) != 0u) {
            specular_transmission *= textureSample(pbr_bindings::specular_transmission_texture, pbr_bindings::specular_transmission_sampler, uv).r;
        }
#endif
        pbr_input.material.specular_transmission = specular_transmission;

        var thickness: f32 = material.thickness;
#ifdef PBR_TRANSMISSION_TEXTURES_SUPPORTED
        if ((material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_THICKNESS_TEXTURE_BIT) != 0u) {
            thickness *= textureSample(pbr_bindings::thickness_texture, pbr_bindings::thickness_sampler, uv).g;
        }
#endif
        // scale thickness, accounting for non-uniform scaling (e.g. a “squished” mesh)
        thickness *= length(
            (transpose(mesh[in.instance_index].model) * vec4(pbr_input.N, 0.0)).xyz
        );
        pbr_input.material.thickness = thickness;

        var diffuse_transmission = material.diffuse_transmission;
#ifdef PBR_TRANSMISSION_TEXTURES_SUPPORTED
        if ((material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DIFFUSE_TRANSMISSION_TEXTURE_BIT) != 0u) {
            diffuse_transmission *= textureSample(pbr_bindings::diffuse_transmission_texture, pbr_bindings::diffuse_transmission_sampler, uv).a;
        }
#endif
        pbr_input.material.diffuse_transmission = diffuse_transmission;

        var diffuse_occlusion: vec3<f32> = vec3(1.0);
        var specular_occlusion: f32 = 1.0;
#ifdef VERTEX_UVS
        if ((material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT) != 0u) {
            diffuse_occlusion = vec3(textureSampleBias(pbr_bindings::occlusion_texture, pbr_bindings::occlusion_sampler, uv, view.mip_bias).r);
        }
#endif
#ifdef SCREEN_SPACE_AMBIENT_OCCLUSION
        let ssao = textureLoad(screen_space_ambient_occlusion_texture, vec2<i32>(in.position.xy), 0i).r;
        let ssao_multibounce = gtao_multibounce(ssao, pbr_input.material.base_color.rgb);
        diffuse_occlusion = min(diffuse_occlusion, ssao_multibounce);
        // Use SSAO to estimate the specular occlusion.
        // Lagarde and Rousiers 2014, "Moving Frostbite to Physically Based Rendering"
        specular_occlusion =  saturate(pow(NdotV + ssao, exp2(-16.0 * roughness - 1.0)) - 1.0 + ssao);
#endif
        pbr_input.diffuse_occlusion = diffuse_occlusion;
        pbr_input.specular_occlusion = specular_occlusion;

        // N (normal vector)
#ifndef LOAD_PREPASS_NORMALS
        pbr_input.N = pbr_functions::apply_normal_mapping(
            material.flags,
            pbr_input.world_normal,
            double_sided,
            is_front,
#ifdef VERTEX_TANGENTS
#ifdef STANDARD_MATERIAL_NORMAL_MAP
            in.world_tangent,
#endif
#endif
#ifdef VERTEX_UVS
            uv,
#endif
            view.mip_bias,
        );
#endif

#ifdef LIGHTMAP
        pbr_input.lightmap_light = lightmap(
            in.uv_b,
            material.lightmap_exposure,
            in.instance_index);
#endif
    }

    return pbr_input;
}
