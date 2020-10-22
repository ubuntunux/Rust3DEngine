use ash::{
    vk,
    Device,
};
use ash::version::DeviceV1_0;

use crate::constants;
use crate::vulkan_context::vulkan_context::{
    SwapchainIndexMap
};
use crate::vulkan_context::vulkan_context;


#[derive(Clone)]
pub struct FramebufferDataCreateInfo {
    pub _framebuffer_name: String,
    pub _framebuffer_width: u32,
    pub _framebuffer_height: u32,
    pub _framebuffer_depth: u32,
    pub _framebuffer_sample_count: vk::SampleCountFlags,
    pub _framebuffer_view_port: vk::Viewport,
    pub _framebuffer_scissor_rect: vk::Rect2D,
    pub _framebuffer_color_attachment_formats: Vec<vk::Format>,
    pub _framebuffer_depth_attachment_formats: Vec<vk::Format>,
    pub _framebuffer_resolve_attachment_formats: Vec<vk::Format>,
    pub _framebuffer_image_views: SwapchainIndexMap<Vec<vk::ImageView>>,
    pub _framebuffer_clear_values: Vec<vk::ClearValue>,
}

impl std::fmt::Debug for FramebufferDataCreateInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FramebufferDataCreateInfo")
            .field("_name", &self._framebuffer_name)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct FramebufferData {
    _framebuffer_info: FramebufferDataCreateInfo,
    _framebuffers: SwapchainIndexMap<vk::Framebuffer>,
    _render_pass_begin_infos: SwapchainIndexMap<vk::RenderPassBeginInfo>
}

impl Default for FramebufferDataCreateInfo {
    fn default() -> FramebufferDataCreateInfo {
        FramebufferDataCreateInfo {
            _framebuffer_name: String::from(""),
            _framebuffer_width: 1024,
            _framebuffer_height: 768,
            _framebuffer_depth: 1,
            _framebuffer_sample_count: vk::SampleCountFlags::TYPE_1,
            _framebuffer_view_port: vulkan_context::create_viewport(0, 0, 1024, 768, 0.0, 1.0),
            _framebuffer_scissor_rect: vulkan_context::create_rect_2d(0, 0, 1024, 768),
            _framebuffer_color_attachment_formats: Vec::<vk::Format>::new(),
            _framebuffer_depth_attachment_formats: Vec::<vk::Format>::new(),
            _framebuffer_resolve_attachment_formats: Vec::<vk::Format>::new(),
            _framebuffer_image_views: SwapchainIndexMap::<Vec<vk::ImageView>>::new(),
            _framebuffer_clear_values: Vec::<vk::ClearValue>::new()
        }
    }
}


pub fn create_framebuffer_data(
    device: &Device,
    render_pass: vk::RenderPass,
    framebuffer_data_create_info: &FramebufferDataCreateInfo
) -> FramebufferData {
    log::info!("Create Framebuffers : {:?} {} {} {}",
        framebuffer_data_create_info._framebuffer_name,
        framebuffer_data_create_info._framebuffer_width,
        framebuffer_data_create_info._framebuffer_height,
        framebuffer_data_create_info._framebuffer_depth
    );

    let get_framebuffer_create_info = |index: usize| -> vk::FramebufferCreateInfo {
        vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&framebuffer_data_create_info._framebuffer_image_views[index])
            .width(framebuffer_data_create_info._framebuffer_width)
            .height(framebuffer_data_create_info._framebuffer_height)
            .layers(framebuffer_data_create_info._framebuffer_depth)
            .build()
    };

    unsafe {
        let framebuffers: Vec<vk::Framebuffer> = constants::SWAPCHAIN_IMAGE_INDICES
            .iter()
            .map(|index| {
                device.create_framebuffer(&get_framebuffer_create_info(*index), None).expect("vkCreateFramebuffer failed!")
            }).collect();

        let render_pass_begin_infos: Vec<vk::RenderPassBeginInfo> = framebuffers
            .iter()
            .map(|framebuffer| {
                vk::RenderPassBeginInfo::builder()
                    .render_pass(render_pass)
                    .framebuffer(*framebuffer)
                    .render_area(vulkan_context::create_rect_2d(
                        0,
                        0,
                        framebuffer_data_create_info._framebuffer_width,
                        framebuffer_data_create_info._framebuffer_height
                    ))
                    .clear_values(&framebuffer_data_create_info._framebuffer_clear_values)
                    .build()
            }).collect();

        FramebufferData {
            _framebuffer_info: (*framebuffer_data_create_info).clone(),
            _framebuffers: framebuffers,
            _render_pass_begin_infos: render_pass_begin_infos
        }
    }
}

pub fn destroy_framebuffer_data(device: &Device, framebuffer_data: &FramebufferData) {
    log::info!("Destroy Framebuffers: {:?} {:?}", framebuffer_data._framebuffer_info._framebuffer_name, framebuffer_data._framebuffers);
    unsafe {
        for framebuffer in framebuffer_data._framebuffers.iter() {
            device.destroy_framebuffer(*framebuffer, None);
        }
    }
}