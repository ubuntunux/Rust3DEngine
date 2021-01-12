use ash::{ vk, Device };

use crate::constants;
use crate::renderer::material_instance::{ PipelineBindingData };
use crate::vulkan_context::descriptor::{ self, DescriptorResourceInfo };
use crate::vulkan_context::framebuffer::{ self, FramebufferData, RenderTargetInfo };
use crate::vulkan_context::texture::TextureData;
use crate::vulkan_context::vulkan_context::SwapchainIndexMap;

pub fn create_framebuffer(
    device: &Device,
    pipeline_binding_data: &PipelineBindingData,
    render_target: &TextureData,
    render_target_layer: u32,
    render_target_miplevel: u32,
    clear_value: Option<vk::ClearValue>,
) -> FramebufferData {
    let render_pass_data = pipeline_binding_data._render_pass_pipeline_data._render_pass_data.borrow();
    framebuffer::create_framebuffer_data(
        device,
        render_pass_data._render_pass,
        format!("{}_{}", render_pass_data._render_pass_data_name, render_target._texture_data_name).as_str(),
        framebuffer::create_framebuffer_data_create_info(
            &[RenderTargetInfo {
                _texture_data: render_target,
                _target_layer: render_target_layer,
                _target_mip_level: render_target_miplevel,
                _clear_value: clear_value,
            }],
            &[],
            &[]
        ),
    )
}

pub fn create_framebuffers(
    device: &Device,
    pipeline_binding_data: &PipelineBindingData,
    framebuffer_name: &str,
    color_render_targets: &[RenderTargetInfo],
    depth_render_targets: &[RenderTargetInfo],
    resolve_render_targets: &[RenderTargetInfo],
) -> FramebufferData {
    let render_pass_data = pipeline_binding_data._render_pass_pipeline_data._render_pass_data.borrow();
    framebuffer::create_framebuffer_data(
        device,
        render_pass_data._render_pass,
        format!("{}_{}", render_pass_data._render_pass_data_name, framebuffer_name).as_str(),
        framebuffer::create_framebuffer_data_create_info(color_render_targets, depth_render_targets, resolve_render_targets),
    )
}

pub fn create_framebuffer_2d_array(
    device: &Device,
    pipeline_binding_data: &PipelineBindingData,
    render_target: &TextureData,
    render_target_miplevel: u32,
    clear_value: Option<vk::ClearValue>,
) -> FramebufferData {
    let render_pass_data = pipeline_binding_data._render_pass_pipeline_data._render_pass_data.borrow();
    let render_target_infos: Vec<RenderTargetInfo> = (0..render_target._image_layers).map(|layer|
        RenderTargetInfo {
            _texture_data: render_target,
            _target_layer: layer,
            _target_mip_level: render_target_miplevel,
            _clear_value: clear_value,
        }
    ).collect();
    framebuffer::create_framebuffer_data(
        device,
        render_pass_data._render_pass,
        format!("{}_{}", render_pass_data._render_pass_data_name, render_target._texture_data_name).as_str(),
        framebuffer::create_framebuffer_data_create_info(
            &render_target_infos,
            &[],
            &[]
        ),
    )
}

pub fn create_descriptor_sets(
    device: &Device,
    pipeline_binding_data: &PipelineBindingData,
    descriptor_resource_infos: &[(usize, DescriptorResourceInfo)]
) -> SwapchainIndexMap<vk::DescriptorSet> {
    let pipeline_data = pipeline_binding_data._render_pass_pipeline_data._pipeline_data.borrow();
    let descriptor_data = &pipeline_data._descriptor_data;
    let descriptor_binding_indices: Vec<u32> = descriptor_data._descriptor_data_create_infos.iter().map(|descriptor_data_create_info| {
        descriptor_data_create_info._descriptor_binding_index
    }).collect();
    let mut descriptor_resource_infos_list = pipeline_binding_data._descriptor_resource_infos_list.clone();
    for (descriptor_binding_index, descriptor_resource_info) in descriptor_resource_infos {
        for swapchain_index in constants::SWAPCHAIN_IMAGE_INDICES.iter() {
            descriptor_resource_infos_list[*swapchain_index][*descriptor_binding_index] = descriptor_resource_info.clone();
        }
    }
    let descriptor_sets = descriptor::create_descriptor_sets(device, descriptor_data);
    let _write_descriptor_sets: SwapchainIndexMap<Vec<vk::WriteDescriptorSet>> = descriptor::create_write_descriptor_sets_with_update(
        device,
        &descriptor_sets,
        &descriptor_binding_indices,
        &descriptor_data._descriptor_set_layout_bindings,
        &descriptor_resource_infos_list,
    );
    descriptor_sets
}

pub fn create_framebuffer_and_descriptor_sets(
    device: &Device,
    pipeline_binding_data: &PipelineBindingData,
    render_target: &TextureData,
    render_target_layer: u32,
    render_target_miplevel: u32,
    clear_value: Option<vk::ClearValue>,
    descriptor_resource_infos: &[(usize, DescriptorResourceInfo)],
) -> (FramebufferData, SwapchainIndexMap<vk::DescriptorSet>) {
    let framebuffer_data = create_framebuffer(
        device,
        pipeline_binding_data,
        render_target,
        render_target_layer,
        render_target_miplevel,
        clear_value
    );
    let descriptor_sets = create_descriptor_sets(
        device,
        pipeline_binding_data,
        descriptor_resource_infos
    );
    (framebuffer_data, descriptor_sets)
}

pub fn create_framebuffers_and_descriptor_sets(
    device: &Device,
    pipeline_binding_data: &PipelineBindingData,
    framebuffer_name: &str,
    color_render_targets: &[RenderTargetInfo],
    depth_render_targets: &[RenderTargetInfo],
    resolve_render_targets: &[RenderTargetInfo],
    descriptor_resource_infos: &[(usize, DescriptorResourceInfo)],
) -> (FramebufferData, SwapchainIndexMap<vk::DescriptorSet>) {
    let framebuffer_data = create_framebuffers(
        device,
        pipeline_binding_data,
        framebuffer_name,
        color_render_targets,
        depth_render_targets,
        resolve_render_targets,
    );
    let descriptor_sets = create_descriptor_sets(
        device,
        pipeline_binding_data,
        descriptor_resource_infos
    );
    (framebuffer_data, descriptor_sets)
}
