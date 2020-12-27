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
) -> FramebufferData {
    let render_pass_data = pipeline_binding_data._render_pass_pipeline_data._render_pass_data.borrow();
    framebuffer::create_framebuffer_data(
        device,
        render_pass_data._render_pass,
        format!("{}_{}", render_pass_data._render_pass_data_name, render_target._texture_data_name).as_str(),
        framebuffer::create_framebuffer_data_create_info(
            &[RenderTargetInfo {
                _texture_data: render_target,
                _layer: render_target_layer,
                _mip_level: render_target_miplevel,
                _clear_value: None,
            }],
            &[],
            &[]
        ),
    )
}

pub fn create_descriptor_sets(
    device: &Device,
    pipeline_binding_data: &PipelineBindingData,
    descriptor_binding_index: usize,
    input_texture: &TextureData,
    input_texture_layer: u32,
    input_texture_miplevel: u32,
) -> SwapchainIndexMap<vk::DescriptorSet> {
    let pipeline_data = pipeline_binding_data._render_pass_pipeline_data._pipeline_data.borrow();
    let descriptor_data = &pipeline_data._descriptor_data;
    let descriptor_binding_indices: Vec<u32> = descriptor_data._descriptor_data_create_infos.iter().map(|descriptor_data_create_info| {
        descriptor_data_create_info._descriptor_binding_index
    }).collect();
    let mut descriptor_resource_infos_list = pipeline_binding_data._descriptor_resource_infos_list.clone();
    for swapchain_index in constants::SWAPCHAIN_IMAGE_INDICES.iter() {
        for descriptor_resource_infos in descriptor_resource_infos_list.get_mut(*swapchain_index).iter_mut() {
            if constants::INVALID_LAYER != input_texture_layer || constants::INVALID_MIP_LEVEL != input_texture_miplevel {
                descriptor_resource_infos[descriptor_binding_index] = DescriptorResourceInfo::DescriptorImageInfo(input_texture.get_sub_image_info(input_texture_layer, input_texture_miplevel));
            } else {
                descriptor_resource_infos[descriptor_binding_index] = DescriptorResourceInfo::DescriptorImageInfo(input_texture.get_default_image_info());
            }
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
    descriptor_binding_index: usize,
    input_texture: &TextureData,
    input_texture_layer: u32,
    input_texture_miplevel: u32,
) -> (FramebufferData, SwapchainIndexMap<vk::DescriptorSet>) {
    let framebuffer_data = create_framebuffer(
        device,
        pipeline_binding_data,
        render_target,
        render_target_layer,
        render_target_miplevel
    );
    let descriptor_sets = create_descriptor_sets(
        device,
        pipeline_binding_data,
        descriptor_binding_index,
        input_texture,
        input_texture_layer,
        input_texture_miplevel,
    );
    (framebuffer_data, descriptor_sets)
}