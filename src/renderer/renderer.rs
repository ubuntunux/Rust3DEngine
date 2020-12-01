use std::str::FromStr;
use std::collections::HashMap;
use std::cell::{ Ref, RefMut };
use std::borrow::{Cow};
use std::ffi::CStr;
use std::vec::Vec;
use ash::{
    vk,
    Device,
    Entry,
    Instance,
};
use ash::prelude::VkResult;
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{
    Surface,
    Swapchain,
};
use ash::version::{InstanceV1_0, DeviceV1_0};
use winit;
use winit::dpi;
use winit::window::{
    Window,
    WindowBuilder
};
use winit::event_loop::EventLoop;
use nalgebra::{ Vector2, Vector3, Vector4, Matrix4 };

use crate::application::SceneManagerData;
use crate::constants;
use crate::vulkan_context::{
    buffer,
    command_buffer,
    device,
    queue,
    sync,
    texture,
};

use crate::vulkan_context::buffer::{ BufferDataInfo };
use crate::vulkan_context::descriptor::{ self, DescriptorResourceInfo };
use crate::vulkan_context::framebuffer::FramebufferData;
use crate::vulkan_context::geometry_buffer::{ self, GeometryData };
use crate::vulkan_context::render_pass::{ RenderPassPipelineDataName, RenderPassPipelineData, RenderPassData, PipelineData };
use crate::vulkan_context::swapchain::{ self, SwapchainData };
use crate::vulkan_context::texture::{ TextureCreateInfo, TextureData };
use crate::vulkan_context::vulkan_context::{ RenderFeatures, SwapchainIndexMap, FrameIndexMap };
use crate::renderer::image_sampler::{ self, ImageSamplerData };
use crate::renderer::material_instance::{ PipelineBindingData, MaterialInstanceData };
use crate::renderer::render_target::{ self, RenderTargetType };
use crate::renderer::buffer_data_infos::{self, BufferDataType, BufferDataInfoMap };
use crate::renderer::post_process::{ PostProcessData_SSAO };
use crate::renderer::push_constants::{ PushConstants_StaticRenderObject, PushConstants_SkeletalRenderObject };
use crate::renderer::render_element::{ RenderElementData };
use crate::resource::{ Resources };
use crate::utilities::system::{ self, RcRefCell };

pub type RenderTargetDataMap = HashMap<RenderTargetType, TextureData>;

// NOTE : RenderMode must match with scene_constants.glsl
#[derive(Clone, Debug, Copy)]
#[allow(non_camel_case_types)]
pub enum RenderMode {
    RenderMode_Common = 0,
    RenderMode_Shadow = 1,
}

// NOTE : RenderObjectType must match with scene_constants.glsl
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum RenderObjectType {
    Static = 0,
    Skeletal = 1,
}

pub unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;
    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };
    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };
    println!(
        "[{:?}]:{:?} [{} ({})] : {}",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );
    vk::FALSE
}

pub fn get_debug_message_level(debug_message_level: vk::DebugUtilsMessageSeverityFlagsEXT) -> vk::DebugUtilsMessageSeverityFlagsEXT {
    match debug_message_level {
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => (
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
        ),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => (
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
        ),
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => (
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
        ),
        _ => (
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
                vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
        ),
    }
}

pub struct RendererData {
    _frame_index: i32,
    _swapchain_index: u32,
    _need_recreate_swapchain: bool,
    _is_first_resize_event: bool,
    pub _window: Window,
    pub _entry: Entry,
    pub _instance: Instance,
    pub _device: Device,
    pub _device_properties: vk::PhysicalDeviceProperties,
    pub _device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub _physical_device: vk::PhysicalDevice,
    pub _surface: vk::SurfaceKHR,
    pub _surface_interface: Surface,
    pub _swapchain_data: swapchain::SwapchainData,
    pub _swapchain_support_details: swapchain::SwapchainSupportDetails,
    pub _swapchain_interface: Swapchain,
    pub _debug_util_interface: DebugUtils,
    pub _debug_call_back: vk::DebugUtilsMessengerEXT,
    pub _image_available_semaphores: FrameIndexMap<vk::Semaphore>,
    pub _render_finished_semaphores: FrameIndexMap<vk::Semaphore>,
    pub _queue_family_datas: queue::QueueFamilyDatas,
    pub _frame_fences: Vec<vk::Fence>,
    pub _command_pool: vk::CommandPool,
    pub _command_buffers: SwapchainIndexMap<vk::CommandBuffer>,
    pub _render_features: RenderFeatures,
    pub _image_samplers: ImageSamplerData,
    pub _debug_render_target: RenderTargetType,
    pub _render_target_data_map: RenderTargetDataMap,
    pub _buffer_data_info_map: BufferDataInfoMap,
    pub _postprocess_ssao: PostProcessData_SSAO,
    pub _resources: RcRefCell<Resources>
}

pub fn create_renderer_data<T>(
    app_name: &str,
    app_version: u32,
    (window_width, window_height): (u32, u32),
    event_loop: &EventLoop<T>,
    resources: RcRefCell<Resources>
) -> RcRefCell<RendererData> {
    unsafe {
        log::info!("create_renderer_data: {}, width: {}, height: {}", constants::ENGINE_NAME, window_width, window_height);
        let window = WindowBuilder::new()
            .with_title(app_name)
            .with_inner_size(dpi::Size::Physical(dpi::PhysicalSize { width: window_width, height: window_height }))
            .build(&event_loop)
            .unwrap();
        let entry = Entry::new().unwrap();
        let surface_extensions = ash_window::enumerate_required_extensions(&window).unwrap();
        let instance: Instance = device::create_vk_instance(&entry, &app_name, app_version, &surface_extensions);
        let surface = device::create_vk_surface(&entry, &instance, &window);
        let surface_interface = Surface::new(&entry, &instance);
        let (physical_device, swapchain_support_details, physical_device_features) = device::select_physical_device(&instance, &surface_interface, surface).unwrap();
        let device_properties: vk::PhysicalDeviceProperties = instance.get_physical_device_properties(physical_device);
        let device_memory_properties: vk::PhysicalDeviceMemoryProperties = instance.get_physical_device_memory_properties(physical_device);
        let msaa_samples = device::get_max_usable_sample_count(&device_properties);
        let queue_family_indices = queue::get_queue_family_indices(
            &instance,
            &surface_interface,
            surface,
            physical_device,
            constants::IS_CONCURRENT_MODE
        );
        let render_features = RenderFeatures {
            _physical_device_features: physical_device_features.clone(),
            _msaa_samples: msaa_samples,
        };
        let graphics_queue_index = queue_family_indices._graphics_queue_index;
        let present_queue_index = queue_family_indices._present_queue_index;
        let queue_family_index_set: Vec<u32> = if graphics_queue_index == present_queue_index {
            vec![graphics_queue_index]
        } else {
            vec![graphics_queue_index, present_queue_index]
        };
        let device = device::create_device(&instance, physical_device, &render_features, &queue_family_index_set);
        let queue_map = queue::create_queues(&device, &queue_family_index_set);
        let default_queue: &vk::Queue = queue_map.get(&queue_family_index_set[0]).unwrap();
        let queue_family_datas = queue::QueueFamilyDatas {
            _graphics_queue: queue_map.get(&graphics_queue_index).unwrap_or(default_queue).clone(),
            _present_queue: queue_map.get(&present_queue_index).unwrap_or(default_queue).clone(),
            _queue_family_index_list: queue_family_index_set.clone(),
            _queue_family_count: queue_map.len() as u32,
            _queue_family_indices: queue_family_indices.clone()
        };
        let swapchain_interface = Swapchain::new(&instance, &device);
        let swapchain_data: swapchain::SwapchainData = swapchain::create_swapchain_data(
            &device,
            &swapchain_interface,
            surface,
            &swapchain_support_details,
            &queue_family_datas,
            constants::ENABLE_IMMEDIATE_MODE
        );
        let image_available_semaphores = sync::create_semaphores(&device);
        let render_finished_semaphores = sync::create_semaphores(&device);
        let frame_fences = sync::create_fences(&device);
        let command_pool = command_buffer::create_command_pool(&device, &queue_family_datas);
        let command_buffers = command_buffer::create_command_buffers(&device, command_pool, constants::SWAPCHAIN_IMAGE_COUNT as u32);

        // debug utils
        let debug_message_level = get_debug_message_level(constants::DEBUG_MESSAGE_LEVEL);
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: debug_message_level,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::all(),
            pfn_user_callback: Some(vulkan_debug_callback),
            ..Default::default()
        };
        let debug_util_interface = DebugUtils::new(&entry, &instance);
        let debug_call_back = debug_util_interface.create_debug_utils_messenger(&debug_info, None).unwrap();
        let mut renderer_data = RendererData {
            _frame_index: 0,
            _swapchain_index: 0,
            _need_recreate_swapchain: false,
            _is_first_resize_event: true,
            _window: window,
            _entry: entry,
            _instance: instance,
            _device: device,
            _device_properties: device_properties,
            _device_memory_properties: device_memory_properties,
            _physical_device: physical_device,
            _surface: surface,
            _surface_interface: surface_interface,
            _swapchain_data: swapchain_data,
            _swapchain_support_details: swapchain_support_details,
            _swapchain_interface: swapchain_interface,
            _debug_util_interface: debug_util_interface,
            _debug_call_back: debug_call_back,
            _image_available_semaphores: image_available_semaphores,
            _render_finished_semaphores: render_finished_semaphores,
            _queue_family_datas: queue_family_datas,
            _frame_fences: frame_fences,
            _command_pool: command_pool,
            _command_buffers: command_buffers,
            _render_features: render_features,
            _image_samplers: ImageSamplerData::default(),
            _debug_render_target: RenderTargetType::BackBuffer,
            _render_target_data_map: RenderTargetDataMap::new(),
            _buffer_data_info_map: BufferDataInfoMap::new(),
            _postprocess_ssao: PostProcessData_SSAO::default(),
            _resources: resources.clone(),
        };

        renderer_data.initialize_renderer();

        system::newRcRefCell(renderer_data)
    }
}

impl RendererData {
    pub fn get_need_recreate_swapchain(&self) -> bool { self._need_recreate_swapchain }
    pub fn set_need_recreate_swapchain(&mut self, value: bool) { self._need_recreate_swapchain = value; }
    pub fn get_is_first_resize_event(&self) -> bool { self._is_first_resize_event }
    pub fn set_is_first_resize_event(&mut self, value: bool) { self._is_first_resize_event = value; }
    pub fn get_instance(&self) -> &Instance { &self._instance }
    pub fn get_device(&self) -> &Device { &self._device }
    pub fn get_device_properties(&self) -> &vk::PhysicalDeviceProperties { &self._device_properties }
    pub fn get_device_memory_properties(&self) -> &vk::PhysicalDeviceMemoryProperties { &self._device_memory_properties }
    pub fn get_physical_device(&self) -> vk::PhysicalDevice { self._physical_device }
    pub fn get_swap_chain_data(&self) -> &SwapchainData { &self._swapchain_data }
    pub fn get_swap_chain_image_views(&self) -> &SwapchainIndexMap<vk::ImageView> { &self._swapchain_data._swapchain_image_views }
    pub fn get_swap_chain_support_details(&self) -> &swapchain::SwapchainSupportDetails { &self._swapchain_support_details }
    pub fn get_swap_chain_index(&self) -> u32 { self._swapchain_index }
    pub fn get_command_pool(&self) -> vk::CommandPool { self._command_pool }
    pub fn get_command_buffers(&self) -> &SwapchainIndexMap<vk::CommandBuffer> { &self._command_buffers }
    pub fn get_command_buffer(&self, index: usize) -> vk::CommandBuffer { self._command_buffers[index] }
    pub fn get_current_command_buffer(&self) -> vk::CommandBuffer { self._command_buffers[self._swapchain_index as usize] }
    pub fn get_graphics_queue(&self) -> vk::Queue { self._queue_family_datas._graphics_queue }
    pub fn get_present_queue(&self) -> vk::Queue { self._queue_family_datas._present_queue }
    pub fn get_buffer_data_info(&self, buffer_data_type: BufferDataType) -> &BufferDataInfo {
        &self._buffer_data_info_map.get(&buffer_data_type).unwrap()
    }

    pub fn next_debug_render_target(&mut self) {
        self._debug_render_target = if RenderTargetType::MaxBound == self._debug_render_target {
            unsafe { std::mem::transmute(0) }
        } else {
            let enum_to_int: i32 = self._debug_render_target.clone() as i32;
            unsafe { std::mem::transmute(enum_to_int + 1) }
        };
        log::info!("Current DebugRenderTarget: {:?}", self._debug_render_target);
    }

    pub fn prev_debug_render_target(&mut self) {
        let enum_to_int: i32 = self._debug_render_target.clone() as i32;
        self._debug_render_target = if 0 == enum_to_int {
            unsafe { std::mem::transmute(RenderTargetType::MaxBound as i32 - 1) }
        } else {
            unsafe { std::mem::transmute(enum_to_int - 1) }
        };
        log::info!("Current DebugRenderTarget: {:?}", self._debug_render_target);
    }

    pub fn get_render_target(&self, render_target_type: RenderTargetType) -> &TextureData {
        &self._render_target_data_map.get(&render_target_type).unwrap()
    }

    pub fn create_render_targets(&mut self) {
        let render_taget_create_infos = render_target::get_render_target_create_infos(self);
        for render_taget_create_info in render_taget_create_infos.iter() {
            let render_target_type: RenderTargetType = RenderTargetType::from_str(render_taget_create_info._texture_name.as_str()).unwrap();
            let texture_data = self.create_render_target(render_taget_create_info);
            self._render_target_data_map.insert(render_target_type, texture_data);
        }
    }

    pub fn create_render_target<T>(&self, texture_create_info: &TextureCreateInfo<T>) -> TextureData {
        texture::create_render_target(
            self.get_instance(),
            self.get_device(),
            self.get_physical_device(),
            self.get_device_memory_properties(),
            self.get_command_pool(),
            self.get_graphics_queue(),
            texture_create_info
        )
    }

    pub fn create_texture<T: Copy>(&self, texture_create_info: &TextureCreateInfo<T>) -> TextureData {
        texture::create_texture_data(
            self.get_instance(),
            self.get_device(),
            self.get_physical_device(),
            self.get_device_memory_properties(),
            self.get_command_pool(),
            self.get_graphics_queue(),
            texture_create_info
        )
    }

    pub fn destroy_texture(&self, texture_data: &TextureData) {
        texture::destroy_texture_data(self.get_device(), texture_data);
    }

    pub fn destroy_render_targets(&mut self) {
        for render_target_data in self._render_target_data_map.values() {
            texture::destroy_texture_data(self.get_device(), render_target_data);
        }
        self._render_target_data_map.clear();
    }

    pub fn destroy_uniform_buffers(&mut self) {
        for buffer_data_info in self._buffer_data_info_map.values() {
            buffer::destroy_buffer_data_info(self.get_device(), buffer_data_info);
        }
        self._buffer_data_info_map.clear();
    }

    pub fn create_geometry_buffer(
        &self,
        geometry_name: &String,
        geometry_create_info: &geometry_buffer::GeometryCreateInfo
    ) -> geometry_buffer::GeometryData {
        geometry_buffer::create_geometry_data(
            self.get_device(),
            self.get_command_pool(),
            self.get_graphics_queue(),
            self.get_device_memory_properties(),
            geometry_name,
            geometry_create_info
        )
    }

    pub fn destroy_geomtry_buffer(&self, geometry_data: &geometry_buffer::GeometryData) {
        geometry_buffer::destroy_geometry_data(self.get_device(), geometry_data);
    }

    pub fn destroy_renderer_data(&mut self) {
        unsafe {
            self.destroy_uniform_buffers();
            image_sampler::destroy_image_samplers(self.get_device(), &self._image_samplers);
            self.destroy_render_targets();
            sync::destroy_semaphores(&self._device, &self._image_available_semaphores);
            sync::destroy_semaphores(&self._device, &self._render_finished_semaphores);
            sync::destroy_fences(&self._device, &self._frame_fences);
            command_buffer::destroy_command_buffers(&self._device, self._command_pool, &self._command_buffers);
            command_buffer::destroy_command_pool(&self._device, self._command_pool);
            swapchain::destroy_swapchain_data(&self._device, &self._swapchain_interface, &self._swapchain_data);
            device::destroy_device(&self._device);
            device::destroy_vk_surface(&self._surface_interface, self._surface);
            self._debug_util_interface.destroy_debug_utils_messenger(self._debug_call_back, None);
            device::destroy_vk_instance(&self._instance);
        }
    }

    pub fn render_pipeline(
        &self,
        command_buffer: vk::CommandBuffer,
        swapchain_index: u32,
        render_pass_pipeline_data_name: &RenderPassPipelineDataName,
        material_instance_name: &String,
        geometry_data: &GeometryData
    ) {
        let resources: Ref<Resources> = self._resources.borrow();
        let material_instance_data: Ref<MaterialInstanceData> = resources.get_material_instance_data(material_instance_name).borrow();
        let pipeline_binding_data = material_instance_data.get_pipeline_binding_data(render_pass_pipeline_data_name);
        let render_pass_data = &pipeline_binding_data._render_pass_pipeline_data._render_pass_data;
        let pipeline_data = &pipeline_binding_data._render_pass_pipeline_data._pipeline_data;
        self.begin_render_pass_pipeline(command_buffer, swapchain_index, render_pass_data, pipeline_data);
        self.bind_descriptor_sets(command_buffer, swapchain_index, pipeline_binding_data);
        self.draw_elements(command_buffer, geometry_data);
        self.end_render_pass(command_buffer);
    }

    pub fn begin_render_pass_pipeline(
        &self,
        command_buffer: vk::CommandBuffer,
        swapchain_index: u32,
        render_pass_data: &RcRefCell<RenderPassData>,
        pipeline_data: &RcRefCell<PipelineData>
    ) {
        let resources: Ref<Resources> = self._resources.borrow();
        let render_pass_data: Ref<RenderPassData> = render_pass_data.borrow();
        let frame_buffer_data: Ref<FramebufferData> = resources.get_framebuffer_data(render_pass_data.get_render_pass_frame_buffer_name()).borrow();
        let render_pass_begin_info = &frame_buffer_data._render_pass_begin_infos[swapchain_index as usize];
        let pipeline_dynamic_states = &pipeline_data.borrow()._pipeline_dynamic_states;
        unsafe {
            self._device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            if pipeline_dynamic_states.contains(&vk::DynamicState::VIEWPORT) {
                self._device.cmd_set_viewport(command_buffer, 0, &[frame_buffer_data._framebuffer_info._framebuffer_view_port]);
            }
            if pipeline_dynamic_states.contains(&vk::DynamicState::SCISSOR) {
                self._device.cmd_set_scissor(command_buffer, 0, &[frame_buffer_data._framebuffer_info._framebuffer_scissor_rect]);
            }
            self._device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline_data.borrow()._pipeline);
        }
    }

    pub fn begin_render_pass_pipeline2(
        &self,
        command_buffer: vk::CommandBuffer,
        swapchain_index: u32,
        render_pass_pipeline_data_name: &RenderPassPipelineDataName
    ) -> RenderPassPipelineData {
        let render_pass_pipeline_data = self._resources.borrow().get_render_pass_pipeline_data(&render_pass_pipeline_data_name);
        self.begin_render_pass_pipeline(command_buffer, swapchain_index, &render_pass_pipeline_data._render_pass_data, &render_pass_pipeline_data._pipeline_data);
        render_pass_pipeline_data
    }

    pub fn bind_descriptor_sets(&self, command_buffer: vk::CommandBuffer, swapchain_index: u32, pipeline_binding_data: &PipelineBindingData) {
        let pipeline_layout = pipeline_binding_data._render_pass_pipeline_data._pipeline_data.borrow()._pipeline_layout;
        let descriptor_set = pipeline_binding_data._descriptor_sets[swapchain_index as usize];
        let dynamic_offsets: &[u32] = &[];
        unsafe {
            self._device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline_layout, 0, &[descriptor_set], dynamic_offsets);
        }
    }

    pub fn update_descriptor_set(
        &self,
        swapchain_index: u32,
        pipeline_binding_data: &mut PipelineBindingData,
        descriptor_offset: usize,
        descriptor_resource_info: &DescriptorResourceInfo
    ) {
        let wirte_descriptor_sets: &mut Vec<vk::WriteDescriptorSet> = &mut pipeline_binding_data._write_descriptor_sets[swapchain_index as usize];
        descriptor::update_write_descriptor_set(wirte_descriptor_sets, descriptor_offset, descriptor_resource_info);
        let wirte_descriptor_set_offset = wirte_descriptor_sets[descriptor_offset];
        let descriptor_copies: &[vk::CopyDescriptorSet] = &[];
        unsafe {
            self._device.update_descriptor_sets(&[wirte_descriptor_set_offset], descriptor_copies);
        }
    }

    pub fn upload_push_constant_data<T>(&self, command_buffer: vk::CommandBuffer, pipeline_data: &PipelineData, push_constant_data: &T) {
        let constants: &[u8] = system::to_bytes(push_constant_data);
        unsafe {
            self._device.cmd_push_constants(command_buffer, pipeline_data._pipeline_layout, vk::ShaderStageFlags::ALL, 0, constants);
        }
    }

    pub fn draw_elements(&self, command_buffer: vk::CommandBuffer, geometry_data: &GeometryData) {
        unsafe {
            let offsets: &[vk::DeviceSize] = &[0];
            const INSTANCE_COUNT: u32 = 1;
            const FIRST_INDEX: u32 = 0;
            const VERTEX_OFFSET: i32 = 0;
            const FIRST_INSTANCE: u32 = 0;
            self._device.cmd_bind_vertex_buffers(command_buffer, 0, &[geometry_data._vertex_buffer_data._buffer], offsets);
            self._device.cmd_bind_index_buffer(command_buffer, geometry_data._index_buffer_data._buffer, 0, vk::IndexType::UINT32);
            self._device.cmd_draw_indexed(command_buffer, geometry_data._vertex_index_count, INSTANCE_COUNT, FIRST_INDEX, VERTEX_OFFSET, FIRST_INSTANCE);
        }
    }

    pub fn end_render_pass(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self._device.cmd_end_render_pass(command_buffer);
        }
    }

    pub fn device_wait_idle(&self) {
        unsafe {
            self._device.device_wait_idle().expect("vkDeviceWaitIdle failed!");
        }
    }

    pub fn initialize_renderer(&mut self) {
        self._swapchain_index = 0;
        self._frame_index = 0;
        self._need_recreate_swapchain = false;
        buffer_data_infos::regist_buffer_data_infos(
            &self._device,
            &self._device_memory_properties,
            &mut self._buffer_data_info_map
        );
        self._image_samplers = image_sampler::create_image_samplers(self.get_device());
        self.create_render_targets();
    }

    pub fn resize_window(&mut self) {
        log::info!("<< resizeWindow >>");
        self.device_wait_idle();

        // destroy swapchain & graphics resources
        self._resources.borrow_mut().unload_graphics_datas(self);
        self.destroy_render_targets();

        // recreate swapchain & graphics resources
        self.recreate_swapchain();
        self.create_render_targets();
        self._resources.borrow_mut().load_graphics_datas(self);
    }

    pub fn recreate_swapchain(&mut self) {
        log::info!("<< recreateSwapChain >>");
        command_buffer::destroy_command_buffers(&self._device, self._command_pool, &self._command_buffers);
        swapchain::destroy_swapchain_data(&self._device, &self._swapchain_interface, &self._swapchain_data);

        self._swapchain_support_details = swapchain::query_swapchain_support(&self._surface_interface, self._physical_device, self._surface);
        self._swapchain_data = swapchain::create_swapchain_data(
            &self._device,
            &self._swapchain_interface,
            self._surface,
            &self._swapchain_support_details,
            &self._queue_family_datas,
            constants::ENABLE_IMMEDIATE_MODE
        );
        self._command_buffers = command_buffer::create_command_buffers(&self._device, self._command_pool, constants::SWAPCHAIN_IMAGE_COUNT as u32);
    }

    pub fn present_swapchain(
        &self,
        command_buffers: &[vk::CommandBuffer],
        fence: vk::Fence,
        image_available_semaphore: vk::Semaphore,
        render_finished_semaphore: vk::Semaphore,
    ) -> VkResult<bool> {
        let wait_semaphores = [image_available_semaphore];
        let wait_mask = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [render_finished_semaphore];
        let submit_info = vk::SubmitInfo {
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_mask.as_ptr(),
            command_buffer_count: command_buffers.len() as u32,
            p_command_buffers: command_buffers.as_ptr(),
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        };

        unsafe {
            let fences = &[fence];
            self._device.reset_fences(fences).expect("failed to reset_fences");

            let waiting_for_fence = false;
            self._device.queue_submit(
                self._queue_family_datas._graphics_queue,
                &[submit_info],
                if waiting_for_fence { fence } else { vk::Fence::null() }
            ).expect("vkQueueSubmit failed!");

            if waiting_for_fence {
                self._device.wait_for_fences(fences, true, std::u64::MAX).expect("vkWaitForFences failed!");
            }

            let present_wait_semaphores = [render_finished_semaphore];
            let swapchains = [self._swapchain_data._swapchain];
            let image_indices = [self._swapchain_index];
            let present_info = vk::PresentInfoKHR {
                wait_semaphore_count: present_wait_semaphores.len() as u32,
                p_wait_semaphores: present_wait_semaphores.as_ptr(),
                swapchain_count: swapchains.len() as u32,
                p_swapchains: swapchains.as_ptr(),
                p_image_indices: image_indices.as_ptr(),
                ..Default::default()
            };

            let is_swapchain_suboptimal: VkResult<bool> = self._swapchain_interface.queue_present(self.get_present_queue(), &present_info);
            // waiting
            self._device.device_wait_idle().expect("failed to device_wait_idle");
            is_swapchain_suboptimal
        }
    }

    pub fn upload_buffer_data_info<T>(&self, swapchain_index: u32, buffer_data_type: BufferDataType, upload_data: &T) {
        let buffer_data_info = self.get_buffer_data_info(buffer_data_type);
        let buffer_data = &buffer_data_info._buffers[swapchain_index as usize];
        buffer::upload_buffer_data(&self._device, buffer_data, system::to_bytes(upload_data));
    }

    pub fn upload_buffer_data_infos<T: Copy>(&self, swapchain_index: u32, buffer_data_type: BufferDataType, upload_data: &[T]) {
        let buffer_data_info = self.get_buffer_data_info(buffer_data_type);
        let buffer_data = &buffer_data_info._buffers[swapchain_index as usize];
        buffer::upload_buffer_data(&self._device, buffer_data, upload_data);
    }

    pub fn upload_buffer_data_info_offset<T>(&self, swapchain_index: u32, buffer_data_type: BufferDataType, upload_data: &T, offset: vk::DeviceSize) {
        let buffer_data_info = self.get_buffer_data_info(buffer_data_type);
        let buffer_data = &buffer_data_info._buffers[swapchain_index as usize];
        buffer::upload_buffer_data_offset(&self._device, buffer_data, system::to_bytes(upload_data), offset);
    }

    pub fn upload_buffer_data_infos_offset<T: Copy>(&self, swapchain_index: u32, buffer_data_type: BufferDataType, upload_data: &[T], offset: vk::DeviceSize) {
        let buffer_data_info = self.get_buffer_data_info(buffer_data_type);
        let buffer_data = &buffer_data_info._buffers[swapchain_index as usize];
        buffer::upload_buffer_data_offset(&self._device, buffer_data, upload_data, offset);
    }

    pub fn render_scene(&mut self, scene_manager: RefMut<SceneManagerData>, elapsed_time: f64, delta_time: f64) {
        unsafe {
            // frame index
            let frame_index = self._frame_index as usize;
            let frame_fence = self._frame_fences[frame_index];
            let image_available_semaphore = self._image_available_semaphores[frame_index];
            let render_finished_semaphore = self._render_finished_semaphores[frame_index];

            // Begin Render
            let (swapchain_index, is_swapchain_suboptimal) = self._swapchain_interface.acquire_next_image(
                self._swapchain_data._swapchain,
                std::u64::MAX,
                image_available_semaphore,
                vk::Fence::null()
            ).unwrap();

            self._swapchain_index = swapchain_index;

            let command_buffer = self._command_buffers[swapchain_index as usize];
            let present_result: vk::Result = if false == is_swapchain_suboptimal {
                let resources = self._resources.borrow();
                let main_camera =  scene_manager.get_main_camera().borrow();
                let main_light = scene_manager.get_main_light().borrow();
                let quad_mesh = resources.get_mesh_data(&String::from("quad")).borrow();
                let quad_geometry_data: Ref<GeometryData> = quad_mesh.get_default_geometry_data().borrow();

                // Upload Uniform Buffers
                let ssao_constants = &self._postprocess_ssao._ssao_constants;
                let light_constants = main_light.get_light_constants();
                let screen_width = self._swapchain_data._swapchain_extent.width as f32;
                let screen_height = self._swapchain_data._swapchain_extent.height as f32;
                let screen_size: Vector2<f32> = Vector2::new(screen_width, screen_height);
                let scene_constants = buffer_data_infos::SceneConstants {
                    _screen_size: screen_size.clone() as Vector2<f32>,
                    _backbuffer_size: screen_size.clone() as Vector2<f32>,
                    _time: elapsed_time as f32,
                    _delta_time: delta_time as f32,
                    _jitter_frame: 0.0,
                    _scene_constants_dummy0: 0,
                };
                let view_constants = buffer_data_infos::ViewConstants {
                    _view: main_camera._view_matrix.into(),
                    _inv_view: main_camera._inv_view_matrix.into(),
                    _view_origin: main_camera._view_origin_matrix.into(),
                    _inv_view_origin: main_camera._inv_view_origin_matrix.into(),
                    _projection: main_camera._projection_matrix.into(),
                    _inv_projection: main_camera._inv_projection_matrix.into(),
                    _view_projection: main_camera._view_projection_matrix.into(),
                    _inv_view_projection: main_camera._inv_view_projection_matrix.into(),
                    _view_origin_projection: main_camera._view_origin_projection_matrix.into(),
                    _inv_view_origin_projection: main_camera._inv_view_origin_projection_matrix.into(),
                    _view_origin_projection_prev: main_camera._view_origin_projection_matrix_prev.into(),
                    _camera_position: main_camera._transform_object._position.clone() as Vector3<f32>,
                    _viewconstants_dummy0: 0.0,
                    _camera_position_prev: main_camera._transform_object._prev_position.clone() as Vector3<f32>,
                    _viewconstants_dummy1: 0.0,
                    _near_far: Vector2::new(constants::NEAR, constants::FAR),
                    _jitter_delta: Vector2::zeros(),
                    _jitter_offset: Vector2::zeros(),
                    _viewconstants_dummy2: 0.0,
                    _viewconstants_dummy3: 0.0,
                };

                self.upload_buffer_data_info(swapchain_index, BufferDataType::SceneConstants, &scene_constants);
                self.upload_buffer_data_info(swapchain_index, BufferDataType::ViewConstants, &view_constants);
                self.upload_buffer_data_info(swapchain_index, BufferDataType::LightConstants, light_constants);
                self.upload_buffer_data_info(swapchain_index, BufferDataType::SSAOConstants, ssao_constants);

                // Begin command buffer
                let command_buffer_begin_info = vk::CommandBufferBeginInfo {
                    flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
                    ..Default::default()
                };
                self._device.begin_command_buffer(command_buffer, &command_buffer_begin_info).expect("vkBeginCommandBuffer failed!");

                // Render
                let static_render_elements = scene_manager.get_static_render_elements();
                let skeletal_render_elements = scene_manager.get_skeletal_render_elements();
                self.render_shadow(command_buffer, swapchain_index, RenderObjectType::Static, &static_render_elements);
                self.render_shadow(command_buffer, swapchain_index, RenderObjectType::Skeletal, &skeletal_render_elements);
                self.render_solid(command_buffer, swapchain_index, RenderObjectType::Static, &static_render_elements);
                self.render_solid(command_buffer, swapchain_index, RenderObjectType::Skeletal, &skeletal_render_elements);
                self.render_post_process(command_buffer, swapchain_index, &quad_geometry_data);

                // Render Final
                let render_final_material_instance_name = String::from("render_final");
                let render_final_render_pass_pipeline_name = RenderPassPipelineDataName {
                    _render_pass_data_name: String::from("render_final"),
                    _pipeline_data_name: String::from("render_final"),
                };
                self.render_pipeline(
                    command_buffer,
                    swapchain_index,
                    &render_final_render_pass_pipeline_name,
                    &render_final_material_instance_name,
                    &quad_geometry_data
                );

                // Render Debug
                //self._debug_render_target = RenderTargetType::Shadow;
                if RenderTargetType::BackBuffer != self._debug_render_target {
                    let render_debug_material_instance_name = String::from("render_debug");
                    let render_debug_render_pass_pipeline_name = RenderPassPipelineDataName {
                        _render_pass_data_name: String::from("render_debug"),
                        _pipeline_data_name: String::from("render_debug"),
                    };

                    let mut render_debug_material_instance_data: RefMut<MaterialInstanceData> = resources.get_material_instance_data(&render_debug_material_instance_name).borrow_mut();
                    let mut render_debug_pipeline_binding_data = render_debug_material_instance_data.get_pipeline_binding_data_mut(&render_debug_render_pass_pipeline_name);
                    self.begin_render_pass_pipeline(
                        command_buffer,
                        swapchain_index,
                        &render_debug_pipeline_binding_data._render_pass_pipeline_data._render_pass_data,
                        &render_debug_pipeline_binding_data._render_pass_pipeline_data._pipeline_data,
                    );
                    // self.begin_render_pass_pipeline2(command_buffer, swapchain_index, &render_debug_render_pass_pipeline_name);

                    let image_info = self.get_render_target(self._debug_render_target);
                    self.update_descriptor_set(
                        swapchain_index,
                        &mut render_debug_pipeline_binding_data,
                        0,
                        &DescriptorResourceInfo::DescriptorImageInfo(image_info._descriptor_image_info),
                    );

                    self.bind_descriptor_sets(command_buffer, swapchain_index, &render_debug_pipeline_binding_data);
                    self.draw_elements(command_buffer, &quad_geometry_data);
                    self.end_render_pass(command_buffer);
                }

                // End command buffer
                self._device.end_command_buffer(command_buffer).expect("vkEndCommandBuffer failed!");

                // End Render
                let present_result = self.present_swapchain(&[command_buffer], frame_fence, image_available_semaphore, render_finished_semaphore);
                match present_result {
                    Ok(is_swapchain_suboptimal) => if is_swapchain_suboptimal { vk::Result::SUBOPTIMAL_KHR } else { vk::Result::SUCCESS },
                    Err(err) => err,
                }
            } else {
                vk::Result::SUBOPTIMAL_KHR
            };

            if vk::Result::ERROR_OUT_OF_DATE_KHR == present_result || vk::Result::SUBOPTIMAL_KHR == present_result {
                self.set_need_recreate_swapchain(true);
            }

            self._frame_index = (self._frame_index + 1) % (constants::MAX_FRAME_COUNT as i32);
        }
    }

    pub fn render_solid(
        &self,
        command_buffer: vk::CommandBuffer,
        swapchain_index: u32,
        render_object_type: RenderObjectType,
        render_elements: &Vec<RenderElementData>
    ) {
        if 0 == render_elements.len() {
            return;
        }

        let render_pass_pipeline_data_name = match render_object_type {
            RenderObjectType::Static => RenderPassPipelineDataName {
                _render_pass_data_name: String::from("render_pass_static_opaque"),
                _pipeline_data_name: String::from("render_object"),
            },
            RenderObjectType::Skeletal => RenderPassPipelineDataName {
                _render_pass_data_name: String::from("render_pass_skeletal_opaque"),
                _pipeline_data_name: String::from("render_object"),
            },
        };

        for (index, render_element) in render_elements.iter().enumerate() {
            let render_object = render_element._render_object.borrow();
            let material_instance_data = render_element._material_instance_data.borrow();
            let pipeline_binding_data = material_instance_data.get_pipeline_binding_data(&render_pass_pipeline_data_name);
            let render_pass_data = &pipeline_binding_data._render_pass_pipeline_data._render_pass_data;
            let pipeline_data = &pipeline_binding_data._render_pass_pipeline_data._pipeline_data;

            if 0 == index {
                // TEST CODE
                if RenderObjectType::Skeletal == render_object_type {
                    let animation_buffer: &Vec<Matrix4<f32>> = render_object.get_animation_buffer(0);
                    let prev_animation_buffer: &Vec<Matrix4<f32>> = render_object.get_prev_animation_buffer(0);
                    let offset = (std::mem::size_of::<Matrix4<f32>>() * constants::MAX_BONES) as vk::DeviceSize;
                    self.upload_buffer_data_infos(swapchain_index, BufferDataType::BoneMatrices, &prev_animation_buffer);
                    self.upload_buffer_data_infos_offset(swapchain_index, BufferDataType::BoneMatrices, &animation_buffer, offset);
                }

                self.begin_render_pass_pipeline(command_buffer, swapchain_index, render_pass_data, pipeline_data);
            }

            self.bind_descriptor_sets(command_buffer, swapchain_index, &pipeline_binding_data);

            match render_object_type {
                RenderObjectType::Static => {
                    self.upload_push_constant_data(
                        command_buffer,
                        &pipeline_data.borrow(),
                        &PushConstants_StaticRenderObject {
                            _model_matrix: render_object._transform_object.get_matrix().clone() as Matrix4<f32>
                        }
                    );
                },
                RenderObjectType::Skeletal => {
                    self.upload_push_constant_data(
                        command_buffer,
                        &pipeline_data.borrow(),
                        &PushConstants_SkeletalRenderObject {
                            _model_matrix: render_object._transform_object.get_matrix().clone() as Matrix4<f32>
                        }
                    );
                },
            };
            self.draw_elements(command_buffer, &render_element._geometry_data.borrow());
        }
        self.end_render_pass(command_buffer);
    }

    pub fn render_shadow(
        &self,
        command_buffer: vk::CommandBuffer,
        swapchain_index: u32,
        render_object_type: RenderObjectType,
        render_elements: &Vec<RenderElementData>
    ) {
        if 0 == render_elements.len() {
            return;
        }

        let (render_pass_pipeline_data_name, material_instance_name) = match render_object_type {
            RenderObjectType::Static => (
                RenderPassPipelineDataName {
                    _render_pass_data_name: String::from("render_pass_static_shadow"),
                    _pipeline_data_name: String::from("render_object"),
                },
                String::from("render_static_shadow")
            ),
            RenderObjectType::Skeletal => (
                RenderPassPipelineDataName {
                    _render_pass_data_name: String::from("render_pass_skeletal_shadow"),
                    _pipeline_data_name: String::from("render_object"),
                },
                String::from("render_skeletal_shadow")
            )
        };

        let resources = self._resources.borrow();
        let material_instance_data = resources.get_material_instance_data(&material_instance_name).borrow();
        let pipeline_binding_data = material_instance_data.get_pipeline_binding_data(&render_pass_pipeline_data_name);
        let render_pass_data = &pipeline_binding_data._render_pass_pipeline_data._render_pass_data;
        let pipeline_data = &pipeline_binding_data._render_pass_pipeline_data._pipeline_data;

        self.begin_render_pass_pipeline(command_buffer, swapchain_index, render_pass_data, pipeline_data);
        self.bind_descriptor_sets(command_buffer, swapchain_index, &pipeline_binding_data);

        for render_element in render_elements.iter() {
            match render_object_type {
                RenderObjectType::Static => {
                    self.upload_push_constant_data(
                        command_buffer,
                        &pipeline_data.borrow(),
                        &PushConstants_StaticRenderObject {
                            _model_matrix: render_element._render_object.borrow()._transform_object.get_matrix().clone() as Matrix4<f32>
                        }
                    );
                },
                RenderObjectType::Skeletal => {
                    self.upload_push_constant_data(
                        command_buffer,
                        &pipeline_data.borrow(),
                        &PushConstants_SkeletalRenderObject {
                            _model_matrix: render_element._render_object.borrow()._transform_object.get_matrix().clone() as Matrix4<f32>
                        }
                    );
                },
            };
            self.draw_elements(command_buffer, &render_element._geometry_data.borrow());
        }
        self.end_render_pass(command_buffer);
    }

    pub fn render_post_process(
        &self,
        command_buffer: vk::CommandBuffer,
        swapchain_index: u32,
        quad_geometry_data: &GeometryData
    ) {
        // SSAO
        let render_ssao_material_instance_name = String::from("render_ssao");
        let render_ssao_render_pass_pipeline_name = RenderPassPipelineDataName {
            _render_pass_data_name: String::from("render_ssao"),
            _pipeline_data_name: String::from("render_ssao"),
        };
        self.render_pipeline(
            command_buffer,
            swapchain_index,
            &render_ssao_render_pass_pipeline_name,
            &render_ssao_material_instance_name,
            &quad_geometry_data
        );

        // Composite GBuffer
        let render_composite_gbuffer_material_instance_name = String::from("composite_gbuffer");
        let render_composite_gbuffer_render_pass_pipeline_name = RenderPassPipelineDataName {
            _render_pass_data_name: String::from("composite_gbuffer"),
            _pipeline_data_name: String::from("composite_gbuffer"),
        };
        self.render_pipeline(
            command_buffer,
            swapchain_index,
            &render_composite_gbuffer_render_pass_pipeline_name,
            &render_composite_gbuffer_material_instance_name,
            &quad_geometry_data
        );

        // Motion Blur
        let render_motion_blur_material_instance_name = String::from("render_motion_blur");
        let render_motion_blur_render_pass_pipeline_name = RenderPassPipelineDataName {
            _render_pass_data_name: String::from("render_motion_blur"),
            _pipeline_data_name: String::from("render_motion_blur"),
        };
        self.render_pipeline(
            command_buffer,
            swapchain_index,
            &render_motion_blur_render_pass_pipeline_name,
            &render_motion_blur_material_instance_name,
            &quad_geometry_data
        );
    }
}