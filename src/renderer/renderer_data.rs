use rand;
use nalgebra::{ Vector3, Vector4 };
use ash::{ vk, Device };

use crate::constants;
use crate::renderer::push_constants::{ PushConstant_BloomHighlight };
use crate::renderer::RenderTargetType;
use crate::renderer::shader_buffer_datas::{ SSAOConstants };
use crate::renderer::utility;
use crate::resource::Resources;
use crate::vulkan_context::buffer::ShaderBufferData;
use crate::vulkan_context::descriptor::{ DescriptorResourceInfo };
use crate::vulkan_context::framebuffer::{ self, FramebufferData, RenderTargetInfo };
use crate::vulkan_context::texture::TextureData;
use crate::vulkan_context::vulkan_context::{self, SwapchainArray};
use crate::utilities::system::RcRefCell;

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct RendererData_LightProbe {
    pub _framebuffer_data: FramebufferData,
    pub _descriptor_sets: SwapchainArray<vk::DescriptorSet>,
}

impl Default for RendererData_LightProbe {
    fn default() -> RendererData_LightProbe {
        RendererData_LightProbe {
            _framebuffer_data: FramebufferData::default(),
            _descriptor_sets: SwapchainArray::new(),
        }
    }
}

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct RendererData_ClearRenderTargets {
    pub _framebuffer_datas: Vec<FramebufferData>,
}

impl Default for RendererData_ClearRenderTargets {
    fn default() -> RendererData_ClearRenderTargets {
        RendererData_ClearRenderTargets {
            _framebuffer_datas: Vec::new()
        }
    }
}

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct RendererData_CompositeGBuffer {
    pub _descriptor_sets0: SwapchainArray<vk::DescriptorSet>,
    pub _descriptor_sets1: SwapchainArray<vk::DescriptorSet>,
}

impl Default for RendererData_CompositeGBuffer {
    fn default() -> RendererData_CompositeGBuffer {
        RendererData_CompositeGBuffer {
            _descriptor_sets0: Vec::new(),
            _descriptor_sets1: Vec::new(),
        }
    }
}

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct RendererData_SceneColorDownSampling {
    pub _dispatch_group_x: u32,
    pub _dispatch_group_y: u32,
    pub _descriptor_sets: Vec<SwapchainArray<vk::DescriptorSet>>,
}

impl Default for RendererData_SceneColorDownSampling {
    fn default() -> RendererData_SceneColorDownSampling {
        RendererData_SceneColorDownSampling {
            _dispatch_group_x: 1,
            _dispatch_group_y: 1,
            _descriptor_sets: Vec::new(),
        }
    }
}

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct RendererData_Bloom {
    pub _bloom_downsample_framebuffer_datas: Vec<FramebufferData>,
    pub _bloom_downsample_descriptor_sets: Vec<SwapchainArray<vk::DescriptorSet>>,
    pub _bloom_temp_framebuffer_datas: Vec<FramebufferData>,
    pub _bloom_temp_descriptor_sets: Vec<SwapchainArray<vk::DescriptorSet>>,
    pub _bloom_blur_framebuffer_datas: Vec<*const FramebufferData>,
    pub _bloom_blur_descriptor_sets: Vec<*const SwapchainArray<vk::DescriptorSet>>,
    pub _bloom_push_constants: PushConstant_BloomHighlight,
}

impl Default for RendererData_Bloom {
    fn default() -> RendererData_Bloom {
        RendererData_Bloom {
            _bloom_downsample_framebuffer_datas: Vec::new(),
            _bloom_downsample_descriptor_sets: Vec::new(),
            _bloom_temp_framebuffer_datas: Vec::new(),
            _bloom_temp_descriptor_sets: Vec::new(),
            _bloom_blur_framebuffer_datas: Vec::new(),
            _bloom_blur_descriptor_sets: Vec::new(),
            _bloom_push_constants: PushConstant_BloomHighlight {
                _bloom_threshold_min: 0.5,
                _bloom_threshold_max: 1000.0,
                _bloom_intensity: 0.25,
                _bloom_scale: 1.0,
            },
        }
    }
}

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct RendererData_SSAO {
    pub _ssao_kernel_size: i32,
    pub _ssao_radius: f32,
    pub _ssao_noise_dim: i32,
    pub _ssao_constants: SSAOConstants,
    pub _ssao_blur_framebuffer_data0: FramebufferData,
    pub _ssao_blur_framebuffer_data1: FramebufferData,
    pub _ssao_blur_descriptor_sets0: SwapchainArray<vk::DescriptorSet>,
    pub _ssao_blur_descriptor_sets1: SwapchainArray<vk::DescriptorSet>,
}

impl Default for RendererData_SSAO {
    fn default() -> RendererData_SSAO {
        let mut random_normals: [Vector4<f32>; 64] = [Vector4::new(0.0, 0.0, 0.0, 0.0); constants::SSAO_KERNEL_SIZE];
        for i in 0..constants::SSAO_KERNEL_SIZE {
            let scale = rand::random::<f32>();
            let normal = Vector3::new(
                rand::random::<f32>() * 2.0 - 1.0,
                rand::random::<f32>() * 0.5 + 0.5,
                rand::random::<f32>() * 2.0 - 1.0
            ).normalize() * scale;
            random_normals[i] = Vector4::new(normal.x, normal.y, normal.z, 0.0);
        }

        RendererData_SSAO {
            _ssao_kernel_size: constants::SSAO_KERNEL_SIZE as i32,
            _ssao_radius: constants::SSAO_RADIUS,
            _ssao_noise_dim: constants::SSAO_NOISE_DIM,
            _ssao_constants: SSAOConstants {
                _ssao_kernel_samples: random_normals
            },
            _ssao_blur_framebuffer_data0: FramebufferData::default(),
            _ssao_blur_framebuffer_data1: FramebufferData::default(),
            _ssao_blur_descriptor_sets0: SwapchainArray::new(),
            _ssao_blur_descriptor_sets1: SwapchainArray::new(),
        }
    }
}

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct RendererData_TAA {
    pub _enable_taa: bool,
    pub _rendertarget_width: u32,
    pub _rendertarget_height: u32,
    pub _taa_resolve_framebuffer_data: FramebufferData,
    pub _taa_descriptor_sets: SwapchainArray<vk::DescriptorSet>,
}

impl Default for RendererData_TAA {
    fn default() -> RendererData_TAA {
        RendererData_TAA {
            _enable_taa: true,
            _rendertarget_width: 1024,
            _rendertarget_height: 768,
            _taa_resolve_framebuffer_data: FramebufferData::default(),
            _taa_descriptor_sets: SwapchainArray::new()
        }
    }
}

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct RendererData_HierachicalMinZ {
    pub _dispatch_group_x: u32,
    pub _dispatch_group_y: u32,
    pub _descriptor_sets: Vec<SwapchainArray<vk::DescriptorSet>>,
}

impl Default for RendererData_HierachicalMinZ {
    fn default() -> RendererData_HierachicalMinZ {
        RendererData_HierachicalMinZ {
            _dispatch_group_x: 1,
            _dispatch_group_y: 1,
            _descriptor_sets: Vec::new(),
        }
    }
}

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct RendererData_SSR {
    pub _framebuffer_data0: FramebufferData,
    pub _framebuffer_data1: FramebufferData,
    pub _descriptor_sets0: SwapchainArray<vk::DescriptorSet>,
    pub _descriptor_sets1: SwapchainArray<vk::DescriptorSet>,
    pub _current_ssr_resolved: RenderTargetType,
    pub _previous_ssr_resolved: RenderTargetType,
}

impl Default for RendererData_SSR {
    fn default() -> RendererData_SSR {
        RendererData_SSR {
            _framebuffer_data0: FramebufferData::default(),
            _framebuffer_data1: FramebufferData::default(),
            _descriptor_sets0: SwapchainArray::new(),
            _descriptor_sets1: SwapchainArray::new(),
            _current_ssr_resolved: RenderTargetType::SSRResolved,
            _previous_ssr_resolved: RenderTargetType::SSRResolvedPrev,
        }
    }
}

impl RendererData_Bloom {
    pub fn initialize(
        &mut self,
        device: &Device,
        resources: &RcRefCell<Resources>,
        render_target_bloom0: &TextureData,
        render_target_bloom_temp0: &TextureData,
    ) {
        let resources = resources.borrow();
        let render_bloom_material_instance = resources.get_material_instance_data("render_bloom").borrow();
        let pipeline_binding_data = render_bloom_material_instance.get_pipeline_binding_data("render_bloom/render_bloom_downsampling");
        let descriptor_binding_index: usize = 0;
        let layer = 0;
        for mip_level in 0..(render_target_bloom0._image_mip_levels - 1) {
            let (bloom_framebuffer_data, bloom_descriptor_set) = utility::create_framebuffer_and_descriptor_sets(
                device, pipeline_binding_data,
                render_target_bloom0, layer, mip_level + 1, None,
                &[(descriptor_binding_index, utility::create_descriptor_image_info_swapchain_array(render_target_bloom0.get_sub_image_info(layer, mip_level)))]
            );
            self._bloom_downsample_framebuffer_datas.push(bloom_framebuffer_data);
            self._bloom_downsample_descriptor_sets.push(bloom_descriptor_set);
        }

        let render_gaussian_blur_material_instance = resources.get_material_instance_data("render_gaussian_blur").borrow();
        let pipeline_binding_data = render_gaussian_blur_material_instance.get_pipeline_binding_data("render_gaussian_blur/render_gaussian_blur");
        let descriptor_binding_index: usize = 0;
        for mip_level in 0..render_target_bloom0._image_mip_levels {
            let (gaussian_blur_h_framebuffer_data, gaussian_blur_h_descriptor_sets) = utility::create_framebuffer_and_descriptor_sets(
                device, pipeline_binding_data,
                render_target_bloom_temp0, layer, mip_level, None,
                &[(descriptor_binding_index, utility::create_descriptor_image_info_swapchain_array(render_target_bloom0.get_sub_image_info(layer, mip_level)))]
            );
            let (gaussian_blur_v_framebuffer_data, gaussian_blur_v_descriptor_sets) = utility::create_framebuffer_and_descriptor_sets(
                device, pipeline_binding_data,
                render_target_bloom0, layer, mip_level, None,
                &[(descriptor_binding_index, utility::create_descriptor_image_info_swapchain_array(render_target_bloom_temp0.get_sub_image_info(layer, mip_level)))]
            );
            self._bloom_temp_framebuffer_datas.push(gaussian_blur_h_framebuffer_data);
            self._bloom_temp_framebuffer_datas.push(gaussian_blur_v_framebuffer_data);
            self._bloom_temp_descriptor_sets.push(gaussian_blur_h_descriptor_sets);
            self._bloom_temp_descriptor_sets.push(gaussian_blur_v_descriptor_sets);
        }

        let render_bloom_framebuffer_data = resources.get_framebuffer_data("render_bloom").as_ptr();
        let render_bloom_hightlight_pipeline_binding_data = render_bloom_material_instance.get_pipeline_binding_data("render_bloom/render_bloom_highlight");

        self._bloom_blur_framebuffer_datas.push(render_bloom_framebuffer_data);
        for framebuffer_data in self._bloom_downsample_framebuffer_datas.iter() {
            self._bloom_blur_framebuffer_datas.push(framebuffer_data);
        }
        self._bloom_blur_descriptor_sets.push(&render_bloom_hightlight_pipeline_binding_data._descriptor_sets);
        for descriptor_set in self._bloom_downsample_descriptor_sets.iter() {
            self._bloom_blur_descriptor_sets.push(descriptor_set);
        }
    }

    pub fn destroy(&mut self, device: &Device) {
        for framebuffer_data in self._bloom_downsample_framebuffer_datas.iter() {
            framebuffer::destroy_framebuffer_data(device, &framebuffer_data);
        }
        for framebuffer_data in self._bloom_temp_framebuffer_datas.iter() {
            framebuffer::destroy_framebuffer_data(device, &framebuffer_data);
        }
        self._bloom_downsample_framebuffer_datas.clear();
        self._bloom_downsample_descriptor_sets.clear();
        self._bloom_temp_framebuffer_datas.clear();
        self._bloom_temp_descriptor_sets.clear();
        self._bloom_blur_framebuffer_datas.clear();
        self._bloom_blur_descriptor_sets.clear();
    }
}

impl RendererData_TAA {
    pub fn initialize(
        &mut self,
        device: &Device,
        resources: &RcRefCell<Resources>,
        taa_render_target: &TextureData,
        taa_resolve_texture: &TextureData,
    ) {
        let resources = resources.borrow();
        let render_copy_material_instance = resources.get_material_instance_data("render_copy").borrow();
        let pipeline_binding_data = render_copy_material_instance.get_pipeline_binding_data("render_copy/render_copy");
        let descriptor_binding_index: usize = 0;
        let layer: u32 = 0;
        let mip_level: u32 = 0;
        let (framebuffer_data, descriptor_sets) = utility::create_framebuffer_and_descriptor_sets(
            device, pipeline_binding_data,
            taa_resolve_texture, layer, mip_level, None,
            &[(descriptor_binding_index, utility::create_descriptor_image_info_swapchain_array(taa_render_target.get_sub_image_info(layer, mip_level)))]
        );
        self._taa_resolve_framebuffer_data = framebuffer_data;
        self._taa_descriptor_sets = descriptor_sets;
        self._rendertarget_width = taa_render_target._image_width;
        self._rendertarget_height = taa_render_target._image_height;
    }

    pub fn destroy(&mut self, device: &Device) {
        framebuffer::destroy_framebuffer_data(device, &self._taa_resolve_framebuffer_data);
        self._taa_descriptor_sets.clear();
    }
}

impl RendererData_SSAO {
    pub fn initialize(
        &mut self,
        device: &Device,
        resources: &RcRefCell<Resources>,
        render_target_ssao: &TextureData,
        render_target_ssao_temp: &TextureData,
    ) {
        let resources = resources.borrow();
        let render_gaussian_blur_material_instance = resources.get_material_instance_data("render_ssao_blur").borrow();
        let pipeline_binding_data = render_gaussian_blur_material_instance.get_pipeline_binding_data("render_ssao_blur/render_ssao_blur");
        let descriptor_binding_index: usize = 0;
        let layer: u32 = 0;
        let mip_level: u32 = 0;
        let (ssao_blur_framebuffer_data0, ssao_blur_descriptor_sets0) = utility::create_framebuffer_and_descriptor_sets(
            device, pipeline_binding_data,
            render_target_ssao_temp, layer, mip_level, None,
            &[(descriptor_binding_index, utility::create_descriptor_image_info_swapchain_array(render_target_ssao.get_sub_image_info(layer, mip_level)))]
        );
        let (ssao_blur_framebuffer_data1, ssao_blur_descriptor_sets1) = utility::create_framebuffer_and_descriptor_sets(
            device, pipeline_binding_data,
            render_target_ssao, layer, mip_level, None,
            &[(descriptor_binding_index, utility::create_descriptor_image_info_swapchain_array(render_target_ssao_temp.get_sub_image_info(layer, mip_level)))]
        );
        self._ssao_blur_framebuffer_data0 = ssao_blur_framebuffer_data0;
        self._ssao_blur_framebuffer_data1 = ssao_blur_framebuffer_data1;
        self._ssao_blur_descriptor_sets0 = ssao_blur_descriptor_sets0;
        self._ssao_blur_descriptor_sets1 = ssao_blur_descriptor_sets1;
    }

    pub fn destroy(&mut self, device: &Device) {
        framebuffer::destroy_framebuffer_data(device, &self._ssao_blur_framebuffer_data0);
        framebuffer::destroy_framebuffer_data(device, &self._ssao_blur_framebuffer_data1);
        self._ssao_blur_descriptor_sets0.clear();
        self._ssao_blur_descriptor_sets1.clear();
    }
}

impl RendererData_HierachicalMinZ {
    pub fn initialize(
        &mut self,
        device: &Device,
        resources: &RcRefCell<Resources>,
        render_target_hierachical_min_z: &TextureData,
    ) {
        let resources = resources.borrow();
        let generate_min_z_material_instance = resources.get_material_instance_data("generate_min_z").borrow();
        let pipeline_binding_data = generate_min_z_material_instance.get_pipeline_binding_data("generate_min_z/generate_min_z");
        let layer: u32 = 0;
        let dispatch_count: u32 = render_target_hierachical_min_z._image_mip_levels - 1;
        for mip_level in 0..dispatch_count {
            let descriptor_sets = utility::create_descriptor_sets(
                device,
                pipeline_binding_data,
                &[
                    (0, utility::create_descriptor_image_info_swapchain_array(render_target_hierachical_min_z.get_sub_image_info(layer, mip_level))),
                    (1, utility::create_descriptor_image_info_swapchain_array(render_target_hierachical_min_z.get_sub_image_info(layer, mip_level + 1))),
                ]
            );
            self._descriptor_sets.push(descriptor_sets);
        }
        self._dispatch_group_x = render_target_hierachical_min_z._image_width;
        self._dispatch_group_y = render_target_hierachical_min_z._image_height;
    }

    pub fn destroy(&mut self, device: &Device) {
        for descriptor_sets in self._descriptor_sets.iter_mut() {
            descriptor_sets.clear();
        }
        self._descriptor_sets.clear();
    }
}

impl RendererData_SceneColorDownSampling {
    pub fn initialize(
        &mut self,
        device: &Device,
        resources: &RcRefCell<Resources>,
        texture_scene_color: &TextureData,
    ) {
        let resources = resources.borrow();
        let downsampling_material_instance = resources.get_material_instance_data("downsampling").borrow();
        let pipeline_binding_data = downsampling_material_instance.get_default_pipeline_binding_data();
        let layer: u32 = 0;
        let dispatch_count: u32 = texture_scene_color._image_mip_levels - 1;
        for mip_level in 0..dispatch_count {
            let descriptor_sets = utility::create_descriptor_sets(
                device,
                pipeline_binding_data,
                &[
                    (0, utility::create_descriptor_image_info_swapchain_array(texture_scene_color.get_sub_image_info(layer, mip_level))),
                    (1, utility::create_descriptor_image_info_swapchain_array(texture_scene_color.get_sub_image_info(layer, mip_level + 1))),
                ]
            );
            self._descriptor_sets.push(descriptor_sets);
        }
        self._dispatch_group_x = texture_scene_color._image_width;
        self._dispatch_group_y = texture_scene_color._image_height;
    }

    pub fn destroy(&mut self, device: &Device) {
        self._descriptor_sets.clear();
    }
}

impl RendererData_SSR {
    pub fn initialize(
        &mut self,
        device: &Device,
        resources: &RcRefCell<Resources>,
        texture_ssr_resolved: &TextureData,
        texture_ssr_resolved_prev: &TextureData,
    ) {
        let resources = resources.borrow();
        let render_copy_material_instance = resources.get_material_instance_data("render_ssr_resolve").borrow();
        let pipeline_binding_data = render_copy_material_instance.get_default_pipeline_binding_data();
        let descriptor_binding_index: usize = 1;
        let layer: u32 = 0;
        let mip_level: u32 = 0;
        let (framebuffer_data0, descriptor_sets0) = utility::create_framebuffer_and_descriptor_sets(
            device, pipeline_binding_data,
            texture_ssr_resolved, layer, mip_level, None,
            &[(descriptor_binding_index, utility::create_descriptor_image_info_swapchain_array(texture_ssr_resolved_prev.get_sub_image_info(layer, mip_level)))]
        );
        let (framebuffer_data1, descriptor_sets1) = utility::create_framebuffer_and_descriptor_sets(
            device, pipeline_binding_data,
            texture_ssr_resolved_prev, layer, mip_level, None,
            &[(descriptor_binding_index, utility::create_descriptor_image_info_swapchain_array(texture_ssr_resolved.get_sub_image_info(layer, mip_level)))]
        );
        self._framebuffer_data0 = framebuffer_data0;
        self._framebuffer_data1 = framebuffer_data1;
        self._descriptor_sets0 = descriptor_sets0;
        self._descriptor_sets1 = descriptor_sets1;
    }

    pub fn update(&mut self) {
        let temp = self._current_ssr_resolved;
        self._current_ssr_resolved = self._previous_ssr_resolved;
        self._previous_ssr_resolved = temp;
    }

    pub fn destroy(&mut self, device: &Device) {
        framebuffer::destroy_framebuffer_data(device, &self._framebuffer_data0);
        framebuffer::destroy_framebuffer_data(device, &self._framebuffer_data1);
    }
}


impl RendererData_CompositeGBuffer {
    pub fn initialize(
        &mut self,
        device: &Device,
        resources: &RcRefCell<Resources>,
        texture_ssr_resolved: &TextureData,
        texture_ssr_resolved_prev: &TextureData,
    ) {
        let resources = resources.borrow();
        let render_copy_material_instance = resources.get_material_instance_data("composite_gbuffer").borrow();
        let pipeline_binding_data = render_copy_material_instance.get_default_pipeline_binding_data();
        let descriptor_binding_index: usize = 11;
        self._descriptor_sets0 = utility::create_descriptor_sets(
            device,
            pipeline_binding_data,
            &[(descriptor_binding_index, utility::create_descriptor_image_info_swapchain_array(texture_ssr_resolved.get_default_image_info()))],
        );
        self._descriptor_sets1 = utility::create_descriptor_sets(
            device, pipeline_binding_data,
            &[(descriptor_binding_index, utility::create_descriptor_image_info_swapchain_array(texture_ssr_resolved_prev.get_default_image_info()))]
        );
    }

    pub fn destroy(&mut self, device: &Device) {
        self._descriptor_sets0.clear();
        self._descriptor_sets1.clear();
    }
}

impl RendererData_ClearRenderTargets {
    pub fn initialize(
        &mut self,
        device: &Device,
        resources: &RcRefCell<Resources>,
        render_targets: &[&TextureData],
    ) {
        let resources = resources.borrow();
        let material_instance = resources.get_material_instance_data("render_color").borrow();
        for render_target in render_targets.iter() {
            let pipeline_binding_data = material_instance.get_pipeline_binding_data(&format!("{:?}/{:?}", render_target._image_format, render_target._image_format));
            for layer in 0..render_target._image_layers {
                for mip_level in 0..render_target._image_mip_levels {
                    self._framebuffer_datas.push(utility::create_framebuffer(device, pipeline_binding_data, render_target, layer, mip_level, None))
                }
            }
        }
    }

    pub fn destroy(&mut self, device: &Device) {
        for framebuffer_data in self._framebuffer_datas.iter() {
            framebuffer::destroy_framebuffer_data(device, framebuffer_data);
        }
        self._framebuffer_datas.clear();
    }
}

impl RendererData_LightProbe {
    pub fn initialize(
        &mut self,
        device: &Device,
        resources: &RcRefCell<Resources>,
        light_probe_color: &TextureData,
        light_probe_atmosphere_color: &TextureData,
        light_probe_atmosphere_inscatter: &TextureData,
        light_probe_depth: &TextureData,
        light_probe_view_constants0: &ShaderBufferData,
    ) {
        let resources = resources.borrow();
        let material_instance = resources.get_material_instance_data("precomputed_atmosphere").borrow();
        let pipeline_binding_data = material_instance.get_pipeline_binding_data("render_atmosphere/default");
        self._framebuffer_data = utility::create_framebuffers(
            device,
            pipeline_binding_data,
            "render_targets_light_probe",
            &[
                RenderTargetInfo { _texture_data: &light_probe_atmosphere_color, _target_layer: 0, _target_mip_level: 0, _clear_value: Some(vulkan_context::get_color_clear_value(0.5, 0.5, 1.0, 0.0)) },
                RenderTargetInfo { _texture_data: &light_probe_atmosphere_inscatter, _target_layer: 0, _target_mip_level: 0, _clear_value: Some(vulkan_context::get_color_clear_value(0.5, 0.5, 1.0, 0.0)) },
                //RenderTargetInfo { _texture_data: &light_probe_depth, _target_layer: 0, _target_mip_level: 0, _clear_value: None },
            ],
            &[],
            &[],
        );

        self._descriptor_sets = utility::create_descriptor_sets(
            device,
            pipeline_binding_data,
            &[
                (1, light_probe_view_constants0._descriptor_buffer_infos.clone())
            ]
        );
    }

    pub fn destroy(&mut self, device: &Device) {
        framebuffer::destroy_framebuffer_data(device, &self._framebuffer_data);
        self._descriptor_sets.clear();
    }
}