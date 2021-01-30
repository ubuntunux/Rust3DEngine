use ash::{
    vk,
};
use crate::renderer::renderer::{
    RenderObjectType,
    RendererData,
};
use crate::resource::render_pass_create_info::{
    clear_render_target,
    clear_framebuffer,
    composite_gbuffer,
    downsampling,
    fft_ocean,
    precomputed_atmosphere,
    generate_min_z,
    render_bloom,
    render_copy,
    render_color,
    render_debug,
    render_final,
    render_forward,
    render_forward_for_light_probe,
    render_gaussian_blur,
    render_motion_blur,
    render_gbuffer,
    render_shadow,
    render_ssao,
    render_ssao_blur,
    render_ssr,
    render_ssr_resolve,
    render_taa,
};
use crate::vulkan_context::render_pass::RenderPassDataCreateInfo;

pub fn get_render_pass_data_create_infos(renderer_data: &RendererData) -> Vec<RenderPassDataCreateInfo> {
    vec![
        clear_render_target::get_render_pass_data_create_info(renderer_data, &[vk::Format::R16G16B16A16_SFLOAT], vk::Format::UNDEFINED),
        clear_render_target::get_render_pass_data_create_info(renderer_data, &[vk::Format::R32_SFLOAT], vk::Format::UNDEFINED),
        clear_render_target::get_render_pass_data_create_info(renderer_data, &[vk::Format::R32G32B32A32_SFLOAT], vk::Format::UNDEFINED),
        clear_render_target::get_render_pass_data_create_info(renderer_data, &[vk::Format::R16G16B16A16_SFLOAT], vk::Format::D32_SFLOAT),
        clear_render_target::get_render_pass_data_create_info(renderer_data, &[], vk::Format::D32_SFLOAT),
        clear_render_target::get_render_pass_data_create_info(
            renderer_data,
            &[vk::Format::R8G8B8A8_UNORM, vk::Format::R8G8B8A8_UNORM, vk::Format::R8G8B8A8_UNORM, vk::Format::R16G16_SFLOAT],
            vk::Format::D32_SFLOAT
        ),
        clear_framebuffer::get_render_pass_data_create_info(renderer_data, "clear_gbuffer"),
        clear_framebuffer::get_render_pass_data_create_info(renderer_data, "clear_shadow"),
        composite_gbuffer::get_render_pass_data_create_info(renderer_data),
        downsampling::get_render_pass_data_create_info(renderer_data),
        fft_ocean::render_fft_init::get_render_pass_data_create_info(renderer_data),
        fft_ocean::render_fft_ocean::get_render_pass_data_create_info(renderer_data),
        fft_ocean::render_fft_variance::get_render_pass_data_create_info(renderer_data),
        fft_ocean::render_fft_waves::get_render_pass_data_create_info(renderer_data),
        precomputed_atmosphere::composite_atmosphere::get_render_pass_data_create_info(renderer_data),
        precomputed_atmosphere::compute_transmittance::get_render_pass_data_create_info(renderer_data),
        precomputed_atmosphere::compute_direct_irradiance::get_render_pass_data_create_info(renderer_data),
        precomputed_atmosphere::compute_indirect_irradiance::get_render_pass_data_create_info(renderer_data),
        precomputed_atmosphere::compute_multiple_scattering::get_render_pass_data_create_info(renderer_data),
        precomputed_atmosphere::compute_single_scattering::get_render_pass_data_create_info(renderer_data),
        precomputed_atmosphere::compute_scattering_density::get_render_pass_data_create_info(renderer_data),
        precomputed_atmosphere::render_atmosphere::get_render_pass_data_create_info(renderer_data),
        generate_min_z::get_render_pass_data_create_info(renderer_data),
        render_bloom::get_render_pass_data_create_info(renderer_data),
        render_copy::get_render_pass_data_create_info(renderer_data),
        render_color::get_render_pass_data_create_info(renderer_data, vk::Format::R16G16B16A16_SFLOAT),
        render_color::get_render_pass_data_create_info(renderer_data, vk::Format::R32_SFLOAT),
        render_color::get_render_pass_data_create_info(renderer_data, vk::Format::R32G32B32A32_SFLOAT),
        render_debug::get_render_pass_data_create_info(renderer_data),
        render_final::get_render_pass_data_create_info(renderer_data),
        render_gaussian_blur::get_render_pass_data_create_info(renderer_data),
        render_motion_blur::get_render_pass_data_create_info(renderer_data),
        render_gbuffer::get_render_pass_data_create_info(renderer_data, RenderObjectType::Skeletal),
        render_gbuffer::get_render_pass_data_create_info(renderer_data, RenderObjectType::Static),
        render_forward::get_render_pass_data_create_info(renderer_data, RenderObjectType::Skeletal),
        render_forward::get_render_pass_data_create_info(renderer_data, RenderObjectType::Static),
        render_shadow::get_render_pass_data_create_info(renderer_data, RenderObjectType::Skeletal),
        render_shadow::get_render_pass_data_create_info(renderer_data, RenderObjectType::Static),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer_data, RenderObjectType::Static, 0),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer_data, RenderObjectType::Static, 1),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer_data, RenderObjectType::Static, 2),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer_data, RenderObjectType::Static, 3),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer_data, RenderObjectType::Static, 4),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer_data, RenderObjectType::Static, 5),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer_data, RenderObjectType::Skeletal, 0),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer_data, RenderObjectType::Skeletal, 1),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer_data, RenderObjectType::Skeletal, 2),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer_data, RenderObjectType::Skeletal, 3),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer_data, RenderObjectType::Skeletal, 4),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer_data, RenderObjectType::Skeletal, 5),
        render_ssao::get_render_pass_data_create_info(renderer_data),
        render_ssao_blur::get_render_pass_data_create_info(renderer_data),
        render_ssr::get_render_pass_data_create_info(renderer_data),
        render_ssr_resolve::get_render_pass_data_create_info(renderer_data),
        render_taa::get_render_pass_data_create_info(renderer_data),
    ]
}