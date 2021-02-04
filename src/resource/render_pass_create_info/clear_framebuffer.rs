use std::path::PathBuf;

use ash::{
    vk,
};

use crate::renderer::renderer::RendererData;
use crate::renderer::render_target::RenderTargetType;
use crate::vulkan_context::framebuffer::{ self, FramebufferDataCreateInfo, RenderTargetInfo };
use crate::vulkan_context::geometry_buffer::{ VertexData };
use crate::vulkan_context::render_pass::{
    RenderPassDataCreateInfo,
    PipelineDataCreateInfo,
    ImageAttachmentDescription,
    DepthStencilStateCreateInfo,
};
use crate::vulkan_context::vulkan_context::{
    self,
    BlendMode,
};


pub fn get_framebuffer_data_create_info(renderer_data: &RendererData, framebuffer_name: &str) -> FramebufferDataCreateInfo {
    let light_probe_depth_only: bool = true;
    match framebuffer_name {
        "clear_gbuffer" => crate::resource::render_gbuffer::get_framebuffer_data_create_info(renderer_data),
        "clear_shadow" => crate::resource::render_shadow::get_framebuffer_data_create_info(renderer_data),
        "clear_capture_height_map" => crate::resource::capture_height_map::get_framebuffer_data_create_info(renderer_data),
        "clear_light_probe_depth_0" => crate::resource::render_forward_for_light_probe::get_framebuffer_data_create_info(renderer_data, 0, light_probe_depth_only),
        "clear_light_probe_depth_1" => crate::resource::render_forward_for_light_probe::get_framebuffer_data_create_info(renderer_data, 1, light_probe_depth_only),
        "clear_light_probe_depth_2" => crate::resource::render_forward_for_light_probe::get_framebuffer_data_create_info(renderer_data, 2, light_probe_depth_only),
        "clear_light_probe_depth_3" => crate::resource::render_forward_for_light_probe::get_framebuffer_data_create_info(renderer_data, 3, light_probe_depth_only),
        "clear_light_probe_depth_4" => crate::resource::render_forward_for_light_probe::get_framebuffer_data_create_info(renderer_data, 4, light_probe_depth_only),
        "clear_light_probe_depth_5" => crate::resource::render_forward_for_light_probe::get_framebuffer_data_create_info(renderer_data, 5, light_probe_depth_only),
        _ => panic!("Not implemented.")
    }
}

pub fn get_render_pass_data_create_info(renderer_data: &RendererData, framebuffer_name: &str) -> RenderPassDataCreateInfo {
    let render_pass_name = String::from(framebuffer_name);
    let framebuffer_data_create_info = get_framebuffer_data_create_info(renderer_data, framebuffer_name);
    let sample_count = framebuffer_data_create_info._framebuffer_sample_count;
    let mut color_attachment_descriptions: Vec<ImageAttachmentDescription> = Vec::new();
    for (i, format) in framebuffer_data_create_info._framebuffer_color_attachment_formats.iter().enumerate() {
        color_attachment_descriptions.push(
            ImageAttachmentDescription {
                _attachment_image_format: *format,
                _attachment_image_samples: sample_count,
                _attachment_load_operation: vk::AttachmentLoadOp::CLEAR,
                _attachment_store_operation: vk::AttachmentStoreOp::STORE,
                _attachment_initial_layout: vk::ImageLayout::UNDEFINED,
                _attachment_final_layout: vk::ImageLayout::GENERAL,
                _attachment_reference_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                ..Default::default()
            }
        );
    }
    let mut depth_attachment_descriptions: Vec<ImageAttachmentDescription> = Vec::new();
    for format in framebuffer_data_create_info._framebuffer_depth_attachment_formats.iter() {
        depth_attachment_descriptions.push(
            ImageAttachmentDescription {
                _attachment_image_format: *format,
                _attachment_image_samples: sample_count,
                _attachment_load_operation: vk::AttachmentLoadOp::CLEAR,
                _attachment_store_operation: vk::AttachmentStoreOp::STORE,
                _attachment_initial_layout: vk::ImageLayout::UNDEFINED,
                _attachment_final_layout: vk::ImageLayout::GENERAL,
                _attachment_reference_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            }
        );
    }
    let subpass_dependencies = vec![
        vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::MEMORY_READ,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::BY_REGION,
        },
        vk::SubpassDependency {
            src_subpass: 0,
            dst_subpass: vk::SUBPASS_EXTERNAL,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_access_mask: vk::AccessFlags::MEMORY_READ,
            dependency_flags: vk::DependencyFlags::BY_REGION,
        }
    ];
    let pipeline_data_create_infos = vec![
        PipelineDataCreateInfo {
            _pipeline_data_create_info_name: String::from("clear"),
            _pipeline_vertex_shader_file: PathBuf::from("render_quad.vert"),
            _pipeline_fragment_shader_file: PathBuf::from("clear_color.frag"),
            _pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            _pipeline_shader_defines: vec![
                format!("ColorAttachmentCount={:?}", color_attachment_descriptions.len() as i32),
            ],
            _pipeline_dynamic_states: vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR],
            _pipeline_sample_count: sample_count,
            _pipeline_color_blend_modes: vec![vulkan_context::get_color_blend_mode(BlendMode::None); color_attachment_descriptions.len()],
            _depth_stencil_state_create_info: DepthStencilStateCreateInfo {
                _depth_write_enable: false,
                ..Default::default()
            },
            _vertex_input_bind_descriptions: VertexData::get_vertex_input_binding_descriptions(),
            _vertex_input_attribute_descriptions: VertexData::create_vertex_input_attribute_descriptions(),
            ..Default::default()
        }
    ];

    RenderPassDataCreateInfo {
        _render_pass_create_info_name: render_pass_name.clone(),
        _render_pass_framebuffer_create_info: framebuffer_data_create_info,
        _color_attachment_descriptions: color_attachment_descriptions,
        _depth_attachment_descriptions: depth_attachment_descriptions,
        _resolve_attachment_descriptions: Vec::new(),
        _subpass_dependencies: subpass_dependencies,
        _pipeline_data_create_infos: pipeline_data_create_infos,
    }
}