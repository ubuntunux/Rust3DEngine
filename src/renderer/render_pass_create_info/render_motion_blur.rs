use std::path::PathBuf;

use ash::{
    vk,
};

use crate::constants;
use crate::utilities::system::{
    enum_to_string
};
use crate::renderer::renderer::{
    RendererData,
};
use crate::renderer::render_target::RenderTargetType;
use crate::renderer::uniform_buffer_data::UniformBufferType;
use crate::vulkan_context::framebuffer::FramebufferDataCreateInfo;
use crate::vulkan_context::render_pass::{
    RenderPassDataCreateInfo,
    PipelineDataCreateInfo,
    ImageAttachmentDescription,
    DepthStencilStateCreateInfo,
};
use crate::vulkan_context::descriptor::{
    DescriptorDataCreateInfo,
    DescriptorResourceType,
};
use crate::vulkan_context::vulkan_context;
use crate::vulkan_context::vulkan_context::{
    BlendMode,
};


pub fn get_framebuffer_data_create_info(
    renderer_data: &RendererData,
    render_pass_name: &String,
) -> FramebufferDataCreateInfo {
    let render_target = renderer_data.get_render_target(RenderTargetType::SceneColorCopy);
    let (width, height) = (render_target._image_width, render_target._image_height);
    FramebufferDataCreateInfo {
        _framebuffer_name: render_pass_name.clone(),
        _framebuffer_width: width,
        _framebuffer_height: height,
        _framebuffer_view_port: vulkan_context::create_viewport(0, 0, width, height, 0.0, 1.0),
        _framebuffer_scissor_rect: vulkan_context::create_rect_2d(0, 0, width, height),
        _framebuffer_color_attachment_formats: vec![render_target._image_format],
        _framebuffer_image_views: vec![vec![render_target._image_view]; constants::SWAPCHAIN_IMAGE_COUNT],
        ..Default::default()
    }
}


pub fn get_render_pass_data_create_info(renderer_data: &RendererData) -> RenderPassDataCreateInfo {
    let render_pass_name = String::from("render_motion_blur");
    let framebuffer_data_create_info = get_framebuffer_data_create_info(renderer_data, &render_pass_name);
    let sample_count = framebuffer_data_create_info._framebuffer_sample_count;
    let mut color_attachment_descriptions: Vec<ImageAttachmentDescription> = Vec::new();
    for format in framebuffer_data_create_info._framebuffer_color_attachment_formats.iter() {
        color_attachment_descriptions.push({
            ImageAttachmentDescription {
                _attachment_image_format: *format,
                _attachment_image_samples: sample_count,
                _attachment_load_operation: vk::AttachmentLoadOp::DONT_CARE,
                _attachment_store_operation: vk::AttachmentStoreOp::STORE,
                _attachment_final_layout: vk::ImageLayout::GENERAL,
                _attachment_reference_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                ..Default::default()
            }
        });
    }
    let subpass_dependencies = vec![
        vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::BY_REGION,
        }
    ];
    let pipeline_data_create_infos = vec![
        PipelineDataCreateInfo {
            _pipeline_data_create_info_name: String::from("render_motion_blur"),
            _pipeline_vertex_shader_file: PathBuf::from("render_quad.vert"),
            _pipeline_fragment_shader_file: PathBuf::from("render_motion_blur.frag"),
            _pipeline_shader_defines: Vec::new(),
            _pipeline_dynamic_states: vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR],
            _pipeline_sample_count: sample_count,
            _pipeline_polygon_mode: vk::PolygonMode::FILL,
            _pipeline_cull_mode: vk::CullModeFlags::NONE,
            _pipeline_front_face: vk::FrontFace::CLOCKWISE,
            _pipeline_viewport: framebuffer_data_create_info._framebuffer_view_port,
            _pipeline_scissor_rect: framebuffer_data_create_info._framebuffer_scissor_rect,
            _pipeline_color_blend_modes: vec![vulkan_context::get_color_blend_mode(BlendMode::None); color_attachment_descriptions.len()],
            _depth_stencil_state_create_info: DepthStencilStateCreateInfo {
                _depth_write_enable: false,
                ..Default::default()
            },
            _push_constant_ranges: Vec::new(),
            _descriptor_data_create_infos: vec![
                DescriptorDataCreateInfo {
                    _descriptor_binding_index: 0,
                    _descriptor_name: enum_to_string(&UniformBufferType::SceneConstants),
                    _descriptor_resource_type: DescriptorResourceType::UniformBuffer,
                    _descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    _descriptor_shader_stage: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                },
                DescriptorDataCreateInfo {
                    _descriptor_binding_index: 1,
                    _descriptor_name: enum_to_string(&UniformBufferType::ViewConstants),
                    _descriptor_resource_type: DescriptorResourceType::UniformBuffer,
                    _descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    _descriptor_shader_stage: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                },
                DescriptorDataCreateInfo {
                    _descriptor_binding_index: 2,
                    _descriptor_name: enum_to_string(&RenderTargetType::SceneColor),
                    _descriptor_resource_type: DescriptorResourceType::RenderTarget,
                    _descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    _descriptor_shader_stage: vk::ShaderStageFlags::FRAGMENT,
                },
                DescriptorDataCreateInfo {
                    _descriptor_binding_index: 3,
                    _descriptor_name: enum_to_string(&RenderTargetType::SceneVelocity),
                    _descriptor_resource_type: DescriptorResourceType::RenderTarget,
                    _descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    _descriptor_shader_stage: vk::ShaderStageFlags::FRAGMENT,
                },
            ],
        }
    ];

    RenderPassDataCreateInfo {
        _render_pass_create_info_name: render_pass_name.clone(),
        _render_pass_frame_buffer_create_info: framebuffer_data_create_info,
        _color_attachment_descriptions: color_attachment_descriptions,
        _depth_attachment_descriptions: Vec::new(),
        _resolve_attachment_descriptions: Vec::new(),
        _subpass_dependencies: subpass_dependencies,
        _pipeline_data_create_infos: pipeline_data_create_infos,
    }
}