use ash::vk;

use rust_engine_3d::vulkan_context::render_pass::RenderPassDataCreateInfo;

use crate::renderer::renderer::RenderObjectType;
use crate::renderer::renderer::Renderer;
use crate::render_pass_create_info::{
    capture_height_map,
    clear_render_target,
    clear_framebuffer,
    composite_gbuffer,
    copy_cube_map,
    downsampling,
    fft_ocean,
    precomputed_atmosphere,
    generate_min_z,
    render_bloom,
    render_copy,
    render_color,
    render_debug,
    render_font,
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
    render_ui,
};

pub fn get_render_pass_data_create_infos(renderer: &Renderer) -> Vec<RenderPassDataCreateInfo> {
    vec![
        clear_render_target::get_render_pass_data_create_info(renderer, &[vk::Format::R16G16B16A16_SFLOAT], vk::Format::UNDEFINED),
        clear_render_target::get_render_pass_data_create_info(renderer, &[vk::Format::R32_SFLOAT], vk::Format::UNDEFINED),
        clear_render_target::get_render_pass_data_create_info(renderer, &[vk::Format::R32G32B32A32_SFLOAT], vk::Format::UNDEFINED),
        clear_render_target::get_render_pass_data_create_info(renderer, &[vk::Format::R16G16B16A16_SFLOAT], vk::Format::D32_SFLOAT),
        clear_render_target::get_render_pass_data_create_info(renderer, &[], vk::Format::D32_SFLOAT),
        clear_render_target::get_render_pass_data_create_info(
            renderer,
            &[vk::Format::R8G8B8A8_UNORM, vk::Format::R8G8B8A8_UNORM, vk::Format::R8G8B8A8_UNORM, vk::Format::R16G16_SFLOAT],
            vk::Format::D32_SFLOAT
        ),
        clear_framebuffer::get_render_pass_data_create_info(renderer, "clear_gbuffer"),
        clear_framebuffer::get_render_pass_data_create_info(renderer, "clear_shadow"),
        clear_framebuffer::get_render_pass_data_create_info(renderer, "clear_capture_height_map"),
        clear_framebuffer::get_render_pass_data_create_info(renderer, "clear_light_probe_depth_0"),
        clear_framebuffer::get_render_pass_data_create_info(renderer, "clear_light_probe_depth_1"),
        clear_framebuffer::get_render_pass_data_create_info(renderer, "clear_light_probe_depth_2"),
        clear_framebuffer::get_render_pass_data_create_info(renderer, "clear_light_probe_depth_3"),
        clear_framebuffer::get_render_pass_data_create_info(renderer, "clear_light_probe_depth_4"),
        clear_framebuffer::get_render_pass_data_create_info(renderer, "clear_light_probe_depth_5"),
        composite_gbuffer::get_render_pass_data_create_info(renderer),
        copy_cube_map::get_render_pass_data_create_info(renderer),
        downsampling::get_render_pass_data_create_info(renderer),
        fft_ocean::render_fft_init::get_render_pass_data_create_info(renderer),
        fft_ocean::render_fft_ocean::get_render_pass_data_create_info(renderer),
        fft_ocean::render_fft_variance::get_render_pass_data_create_info(renderer),
        fft_ocean::render_fft_waves::get_render_pass_data_create_info(renderer),
        precomputed_atmosphere::composite_atmosphere::get_render_pass_data_create_info(renderer),
        precomputed_atmosphere::compute_transmittance::get_render_pass_data_create_info(renderer),
        precomputed_atmosphere::compute_direct_irradiance::get_render_pass_data_create_info(renderer),
        precomputed_atmosphere::compute_indirect_irradiance::get_render_pass_data_create_info(renderer),
        precomputed_atmosphere::compute_multiple_scattering::get_render_pass_data_create_info(renderer),
        precomputed_atmosphere::compute_single_scattering::get_render_pass_data_create_info(renderer),
        precomputed_atmosphere::compute_scattering_density::get_render_pass_data_create_info(renderer),
        precomputed_atmosphere::render_atmosphere::get_render_pass_data_create_info(renderer),
        generate_min_z::get_render_pass_data_create_info(renderer),
        render_bloom::get_render_pass_data_create_info(renderer),
        render_copy::get_render_pass_data_create_info(renderer),
        render_color::get_render_pass_data_create_info(renderer, vk::Format::R16G16B16A16_SFLOAT),
        render_color::get_render_pass_data_create_info(renderer, vk::Format::R32_SFLOAT),
        render_color::get_render_pass_data_create_info(renderer, vk::Format::R32G32B32A32_SFLOAT),
        render_debug::get_render_pass_data_create_info(renderer),
        render_font::get_render_pass_data_create_info(renderer),
        render_final::get_render_pass_data_create_info(renderer),
        render_gaussian_blur::get_render_pass_data_create_info(renderer),
        render_motion_blur::get_render_pass_data_create_info(renderer),
        render_gbuffer::get_render_pass_data_create_info(renderer, RenderObjectType::Skeletal),
        render_gbuffer::get_render_pass_data_create_info(renderer, RenderObjectType::Static),
        render_forward::get_render_pass_data_create_info(renderer, RenderObjectType::Skeletal),
        render_forward::get_render_pass_data_create_info(renderer, RenderObjectType::Static),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer, RenderObjectType::Static, 0),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer, RenderObjectType::Static, 1),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer, RenderObjectType::Static, 2),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer, RenderObjectType::Static, 3),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer, RenderObjectType::Static, 4),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer, RenderObjectType::Static, 5),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer, RenderObjectType::Skeletal, 0),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer, RenderObjectType::Skeletal, 1),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer, RenderObjectType::Skeletal, 2),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer, RenderObjectType::Skeletal, 3),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer, RenderObjectType::Skeletal, 4),
        render_forward_for_light_probe::get_render_pass_data_create_info(renderer, RenderObjectType::Skeletal, 5),
        render_shadow::get_render_pass_data_create_info(renderer, RenderObjectType::Skeletal),
        render_shadow::get_render_pass_data_create_info(renderer, RenderObjectType::Static),
        capture_height_map::get_render_pass_data_create_info(renderer, RenderObjectType::Skeletal),
        capture_height_map::get_render_pass_data_create_info(renderer, RenderObjectType::Static),
        render_ssao::get_render_pass_data_create_info(renderer),
        render_ssao_blur::get_render_pass_data_create_info(renderer),
        render_ssr::get_render_pass_data_create_info(renderer),
        render_ssr_resolve::get_render_pass_data_create_info(renderer),
        render_taa::get_render_pass_data_create_info(renderer),
        render_ui::get_render_pass_data_create_info(renderer),
    ]
}