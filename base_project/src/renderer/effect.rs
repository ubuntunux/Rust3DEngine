use std::collections::HashMap;
use std::cmp::{max, min};
use ash::{
    vk,
    Device
};
use nalgebra::{ Vector3, Vector4, Matrix4 };

use rust_engine_3d::constants::{
    MAX_EMITTER_COUNT,
    MAX_PARTICLE_COUNT,
    PROCESS_GPU_PARTICLE_WORK_GROUP_SIZE,
};
use rust_engine_3d::renderer::effect::*;
use rust_engine_3d::renderer::material_instance::{ PipelineBindingData, MaterialInstanceData };
use rust_engine_3d::renderer::renderer::RendererData;
use rust_engine_3d::resource::resource::Resources;
use rust_engine_3d::vulkan_context::render_pass::{ RenderPassData, PipelineData };
use rust_engine_3d::utilities::system::RcRefCell;

use crate::renderer::push_constants::NONE_PUSH_CONSTANT;
use crate::renderer::renderer::Renderer;
use crate::renderer::shader_buffer_datas::ShaderBufferDataType;

// shader storage buffer
#[derive(Debug, Clone, Copy, Default)]
pub struct GpuParticleStaticConstants {
    pub _spawn_volume_transform: Matrix4<f32>,
    pub _spawn_volume_info: Vector4<f32>,
    pub _rotation_min: Vector3<f32>,
    pub _particle_lifetime_min: f32,
    pub _rotation_max: Vector3<f32>,
    pub _particle_lifetime_max: f32,
    pub _scale_min: Vector3<f32>,
    pub _spawn_volume_type: i32,
    pub _scale_max: Vector3<f32>,
    pub _max_particle_count: i32,
    pub _align_mode: i32,
    pub _geometry_type: i32,
    pub _reserved0: i32,
    pub _reserved1: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GpuParticleDynamicConstants {
    pub _emitter_transform: Matrix4<f32>,
    pub _spawn_count: i32,
    pub _allocated_emitter_index: i32,
    pub _allocated_particle_offset: i32,
    pub _reserved0: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GpuParticleCountBufferData {
    pub _particle_alive_count: i32,
    pub _prev_particle_alive_count: i32,
    pub _particle_dead_count: i32,
    pub _reserved0: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GpuParticleEmitterIndexBufferData {
    pub _emitter_index: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GpuParticleUpdateBufferData {
    pub _particle_emitter_transform: Matrix4<f32>,
    pub _particle_relative_position: Vector3<f32>,
    pub _particle_elapsed_time: f32,
    pub _particle_local_position: Vector3<f32>,
    pub _particle_initial_life_time: f32,
    pub _particle_initial_rotation: Vector3<f32>,
    pub _particle_state: u32,
    pub _particle_initial_scale: Vector3<f32>,
    pub _reserved0: f32,
}

// push constants
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct PushConstant_RenderParticle {
    pub _allocated_emitter_index: i32,
    pub _allocated_particle_offset: i32,
    pub _reserved1: i32,
    pub _reserved2: i32,
}

impl Default for PushConstant_RenderParticle {
    fn default() -> PushConstant_RenderParticle {
        PushConstant_RenderParticle {
            _allocated_emitter_index: 0,
            _allocated_particle_offset: 0,
            _reserved1: 0,
            _reserved2: 0,
        }
    }
}

// interfaces
pub struct EffectManager {
    pub _effect_manager_data: *const EffectManagerData,
    pub _effect_render_group: Vec<*const EmitterInstance>,
    pub _gpu_particle_static_constants: Vec<GpuParticleStaticConstants>,
    pub _gpu_particle_dynamic_constants: Vec<GpuParticleDynamicConstants>,
    pub _gpu_particle_emitter_indices: Vec<i32>,
}

impl EffectManagerBase for EffectManager {
    fn initialize_effect_manager(&mut self, effect_manager_data: *const EffectManagerData) {
        self._effect_manager_data = effect_manager_data;
    }

    fn get_effect_manager_data(&self) -> &EffectManagerData {
        unsafe { &*self._effect_manager_data }
    }

    fn get_effect_manager_data_mut(&self) -> &mut EffectManagerData {
        unsafe { &mut *(self._effect_manager_data as *mut EffectManagerData) }
    }

    fn create_effect(&mut self, effect_create_info: &EffectCreateInfo) -> i64 {
        self.get_effect_manager_data_mut().create_effect(effect_create_info)
    }

    fn get_effect(&self, effect_id: i64) -> Option<&RcRefCell<EffectInstance>> {
        self.get_effect_manager_data().get_effect(effect_id)
    }

    fn get_effects(&self) -> &HashMap<i64, RcRefCell<EffectInstance>> {
        self.get_effect_manager_data().get_effects()
    }

    fn update_effects(&mut self, delta_time: f32) {
        self.get_effect_manager_data_mut().update_effects(delta_time);
    }

    fn gather_render_effects(&mut self) {
        let effects = unsafe { &(*self._effect_manager_data)._effects };
        self._effect_render_group.clear();
        for (_effect_id, effect) in effects.iter() {
            let mut effect = effect.borrow_mut();
            if effect._is_alive {
                for emitter in effect._emitters.iter_mut() {
                    if emitter._is_alive {
                        self._effect_render_group.push(emitter);
                    }
                }
            }
        }
    }
}

impl EffectManager {
    pub fn create_effect_manager() -> Box<EffectManager> {
        Box::new(EffectManager {
            _effect_manager_data: std::ptr::null(),
            _effect_render_group: Vec::new(),
            _gpu_particle_static_constants: unsafe { vec![GpuParticleStaticConstants::default(); MAX_EMITTER_COUNT as usize] },
            _gpu_particle_dynamic_constants: unsafe { vec![GpuParticleDynamicConstants::default(); MAX_EMITTER_COUNT as usize] },
            _gpu_particle_emitter_indices: unsafe { vec![INVALID_ALLOCATED_EMITTER_INDEX; MAX_PARTICLE_COUNT as usize] },
        })
    }

    pub fn get_effect_manager_data(&self) -> &EffectManagerData {
        unsafe { &*self._effect_manager_data }
    }

    pub fn get_effect_manager_data_mut(&self) -> &mut EffectManagerData {
        unsafe { &mut *(self._effect_manager_data as *mut EffectManagerData) }
    }

    pub fn prepare_framebuffer_and_descriptors(&mut self, renderer: &Renderer, resources: &Resources) {
        let device = renderer.get_renderer_data().get_device();
        // let pipeline_binding_data = material_instance.get_pipeline_binding_data("render_fft_waves/render_fft_x");
        // self._fft_wave_x_fft_a_framebuffer = utility::create_framebuffer_2d_array(device, &pipeline_binding_data.get_render_pass_data().borrow(), texture_fft_a, mip_level, None);
        // self._fft_wave_x_fft_b_framebuffer = utility::create_framebuffer_2d_array(device, &pipeline_binding_data.get_render_pass_data().borrow(), texture_fft_b, mip_level, None);
        // let fft_waves_descriptor_binding_index = 1;
        // self._fft_wave_x_fft_a_descriptor_sets = utility::create_descriptor_sets(
        //     device,
        //     pipeline_binding_data,
        //     &[(fft_waves_descriptor_binding_index, utility::create_descriptor_image_info_swapchain_array(texture_fft_a.get_default_image_info()))]
        // );
        // self._fft_wave_x_fft_b_descriptor_sets = utility::create_descriptor_sets(
        //     device,
        //     pipeline_binding_data,
        //     &[(fft_waves_descriptor_binding_index, utility::create_descriptor_image_info_swapchain_array(texture_fft_b.get_default_image_info()))]
        // );
    }

    pub fn destroy_framebuffer_and_descriptors(&mut self, device: &Device) {

    }

    pub fn process_gpu_particles(
        &mut self,
        command_buffer: vk::CommandBuffer,
        swapchain_index: u32,
        renderer: &Renderer,
        resources: &Resources,
    ) {
        let effect_manager_data: &mut EffectManagerData = unsafe { &mut *(self._effect_manager_data as *mut EffectManagerData) };
        let allocated_emitter_count = effect_manager_data._allocated_emitter_count as isize;
        if 0 == allocated_emitter_count {
            return;
        }

        let mut process_emitter_count: i32 = 0;
        let mut process_gpu_particle_count: i32 = 0;
        for emitter_index in 0..allocated_emitter_count {
            let emitter_ptr: *const EmitterInstance = unsafe { *effect_manager_data._allocated_emitters.as_ptr().offset(emitter_index) };
            if emitter_ptr.is_null() {
                continue;
            }

            let emitter: &mut EmitterInstance = unsafe { &mut *(emitter_ptr as *mut EmitterInstance) };
            let emitter_data: &EmitterData = emitter.get_emitter_data();
            let available_particle_count = unsafe { max(0, min(MAX_PARTICLE_COUNT - process_gpu_particle_count, emitter_data._max_particle_count)) };
            if 0 == available_particle_count {
                effect_manager_data.deallocate_emitter(emitter);
                continue;
            }

            let need_to_change_allocate_emitter_index = process_emitter_count != emitter._allocated_emitter_index;
            let need_to_upload_static_constant_buffer = emitter._need_to_upload_static_constant_buffer || need_to_change_allocate_emitter_index;

            // update static constants
            if need_to_upload_static_constant_buffer {
                let gpu_particle_static_constant = &mut self._gpu_particle_static_constants[process_emitter_count as usize];
                gpu_particle_static_constant._spawn_volume_transform.clone_from(&emitter_data._spawn_volume_transform);
                gpu_particle_static_constant._spawn_volume_info.clone_from(&emitter_data._spawn_volume_info);
                gpu_particle_static_constant._spawn_volume_type = emitter_data._spawn_volume_type as i32;
                gpu_particle_static_constant._rotation_min.clone_from(&emitter_data._rotation_min);
                gpu_particle_static_constant._rotation_max.clone_from(&emitter_data._rotation_max);
                gpu_particle_static_constant._particle_lifetime_min = emitter_data._particle_lifetime_min;
                gpu_particle_static_constant._particle_lifetime_max = emitter_data._particle_lifetime_max;
                gpu_particle_static_constant._scale_min.clone_from(&emitter_data._scale_min);
                gpu_particle_static_constant._scale_max.clone_from(&emitter_data._scale_max);
                gpu_particle_static_constant._max_particle_count = emitter_data._max_particle_count;
                gpu_particle_static_constant._align_mode = emitter_data._align_mode as i32;
                gpu_particle_static_constant._geometry_type = emitter_data._geometry_type as i32;
                emitter._need_to_upload_static_constant_buffer = false;
            }

            // update dynamic constants
            let gpu_particle_dynamic_constant = &mut self._gpu_particle_dynamic_constants[process_emitter_count as usize];
            {
                gpu_particle_dynamic_constant._emitter_transform.clone_from(&emitter._emitter_world_transform);
                gpu_particle_dynamic_constant._spawn_count = emitter._particle_spawn_count;
                gpu_particle_dynamic_constant._allocated_emitter_index = process_emitter_count;
                gpu_particle_dynamic_constant._allocated_particle_offset = process_gpu_particle_count;
            }

            if need_to_change_allocate_emitter_index {
                effect_manager_data._allocated_emitters[process_emitter_count as usize] = emitter_ptr;
                effect_manager_data._allocated_emitters[emitter_index as usize] = std::ptr::null();
                emitter._allocated_emitter_index = process_emitter_count;
            }

            // fill gpu particle allocated emitter index
            if need_to_upload_static_constant_buffer || process_gpu_particle_count != emitter._allocated_particle_offset || available_particle_count != emitter._allocated_particle_count {
                for gpu_particle_offset in process_gpu_particle_count..(process_gpu_particle_count + available_particle_count) {
                    self._gpu_particle_emitter_indices[gpu_particle_offset as usize] = process_emitter_count;
                }
                emitter._allocated_particle_offset = process_gpu_particle_count;
                emitter._allocated_particle_count = available_particle_count;
            }

            //
            process_gpu_particle_count += available_particle_count;
            process_emitter_count += 1;
        }

        effect_manager_data._allocated_emitter_count = process_emitter_count;
        effect_manager_data._allocated_particle_count = process_gpu_particle_count;

        if 0 < process_emitter_count {
            // Upload Uniform Buffers
            let gpu_particle_static_constants_buffer = renderer.get_shader_buffer_data(&ShaderBufferDataType::GpuParticleStaticConstants);
            let gpu_particle_dynamic_constants_buffer = renderer.get_shader_buffer_data(&ShaderBufferDataType::GpuParticleDynamicConstants);
            let gpu_particle_emitter_index_buffer = renderer.get_shader_buffer_data(&ShaderBufferDataType::GpuParticleEmitterIndexBuffer);
            let gpu_particle_count_buffer = renderer.get_shader_buffer_data(&ShaderBufferDataType::GpuParticleCountBuffer);
            let gpu_particle_count_buffer_store = renderer.get_shader_buffer_data(&ShaderBufferDataType::GpuParticleCountBufferStore);
            let gpu_particle_update_buffer = renderer.get_shader_buffer_data(&ShaderBufferDataType::GpuParticleUpdateBuffer);
            let gpu_particle_update_buffer_store = renderer.get_shader_buffer_data(&ShaderBufferDataType::GpuParticleUpdateBufferStore);
            renderer.get_renderer_data().upload_shader_buffer_datas(
                command_buffer,
                swapchain_index,
                gpu_particle_static_constants_buffer,
                &self._gpu_particle_static_constants[0..process_emitter_count as usize]
            );
            renderer.get_renderer_data().upload_shader_buffer_datas(
                command_buffer,
                swapchain_index,
                gpu_particle_dynamic_constants_buffer,
                &self._gpu_particle_dynamic_constants[0..process_emitter_count as usize]
            );
            renderer.get_renderer_data().upload_shader_buffer_datas(
                command_buffer,
                swapchain_index,
                gpu_particle_emitter_index_buffer,
                &self._gpu_particle_emitter_indices[0..process_gpu_particle_count as usize]
            );

            //
            let material_instance_data = &resources.get_material_instance_data("process_gpu_particle").borrow();

            // barrier for compute gpu particle count pipeline
            let gpu_particle_static_constants_buffer_data = &gpu_particle_static_constants_buffer._buffers[swapchain_index as usize];
            let gpu_particle_dynamic_constants_buffer_data = &gpu_particle_dynamic_constants_buffer._buffers[swapchain_index as usize];
            let buffer_memory_barriers: [vk::BufferMemoryBarrier; 2] = [
                vk::BufferMemoryBarrier {
                    src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                    dst_access_mask: vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE,
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    buffer: gpu_particle_static_constants_buffer_data._buffer,
                    offset: 0,
                    size: gpu_particle_static_constants_buffer_data._buffer_memory_requirements.size,
                    ..Default::default()
                },
                vk::BufferMemoryBarrier {
                    src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                    dst_access_mask: vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE,
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    buffer: gpu_particle_dynamic_constants_buffer_data._buffer,
                    offset: 0,
                    size: gpu_particle_dynamic_constants_buffer_data._buffer_memory_requirements.size,
                    ..Default::default()
                }
            ];
            renderer.get_renderer_data().pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::default(),
                &[],
                &buffer_memory_barriers,
                &[],
            );

            // compute gpu particle count
            let pipeline_binding_data: &PipelineBindingData = material_instance_data.get_pipeline_binding_data("process_gpu_particle/compute_gpu_particle_count");
            let thread_group_count = (process_emitter_count + PROCESS_GPU_PARTICLE_WORK_GROUP_SIZE - 1) / PROCESS_GPU_PARTICLE_WORK_GROUP_SIZE;
            renderer.get_renderer_data().dispatch_render_pass_pipeline(
                command_buffer,
                swapchain_index,
                pipeline_binding_data,
                thread_group_count as u32,
                1,
                1,
                None,
                NONE_PUSH_CONSTANT,
            );

            // barrier for update gpu particles pipeline
            let gpu_particle_count_buffer_data = &gpu_particle_count_buffer._buffers[swapchain_index as usize];
            let buffer_memory_barriers: [vk::BufferMemoryBarrier; 1] = [
                vk::BufferMemoryBarrier {
                    src_access_mask: vk::AccessFlags::SHADER_WRITE,
                    dst_access_mask: vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE,
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    buffer: gpu_particle_count_buffer_data._buffer,
                    offset: 0,
                    size: gpu_particle_count_buffer_data._buffer_memory_requirements.size,
                    ..Default::default()
                }
            ];
            renderer.get_renderer_data().pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::default(),
                &[],
                &buffer_memory_barriers,
                &[],
            );

            // update gpu particles
            let pipeline_binding_data: &PipelineBindingData = material_instance_data.get_pipeline_binding_data("process_gpu_particle/update_gpu_particle");
            let thread_group_count = (process_gpu_particle_count + PROCESS_GPU_PARTICLE_WORK_GROUP_SIZE - 1) / PROCESS_GPU_PARTICLE_WORK_GROUP_SIZE;
            renderer.get_renderer_data().dispatch_render_pass_pipeline(
                command_buffer,
                swapchain_index,
                pipeline_binding_data,
                thread_group_count as u32,
                1,
                1,
                None,
                NONE_PUSH_CONSTANT,
            );

            // barrier for render gpu particles pipeline
            let gpu_particle_update_buffer_data = &gpu_particle_update_buffer._buffers[swapchain_index as usize];
            let buffer_memory_barriers: [vk::BufferMemoryBarrier; 2] = [
                vk::BufferMemoryBarrier {
                    src_access_mask: vk::AccessFlags::SHADER_WRITE,
                    dst_access_mask: vk::AccessFlags::SHADER_READ,
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    buffer: gpu_particle_count_buffer_data._buffer,
                    offset: 0,
                    size: gpu_particle_count_buffer_data._buffer_memory_requirements.size,
                    ..Default::default()
                },
                vk::BufferMemoryBarrier {
                    src_access_mask: vk::AccessFlags::SHADER_WRITE,
                    dst_access_mask: vk::AccessFlags::SHADER_READ,
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    buffer: gpu_particle_update_buffer_data._buffer,
                    offset: 0,
                    size: gpu_particle_update_buffer_data._buffer_memory_requirements.size,
                    ..Default::default()
                },
            ];
            renderer.get_renderer_data().pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::VERTEX_SHADER,
                vk::DependencyFlags::default(),
                &[],
                &buffer_memory_barriers,
                &[],
            );
        }
    }

    pub fn render_effects(
        &self,
        command_buffer: vk::CommandBuffer,
        swapchain_index: u32,
        renderer: &Renderer,
        resources: &Resources,
    ) {
        if self._effect_render_group.is_empty() {
            return;
        }

        let renderer_data: &RendererData = renderer.get_renderer_data();
        let quad_mesh = resources.get_mesh_data("quad").borrow();
        let quad_geometry_data = quad_mesh.get_default_geometry_data().borrow();
        let render_pass_pipeline_data_name = "render_particle_translucent/alpha_blend";
        let mut prev_pipeline_data: *const PipelineData = std::ptr::null();
        let mut prev_pipeline_binding_data: *const PipelineBindingData = std::ptr::null();
        for emitter in self._effect_render_group.iter() {
            let emitter: &EmitterInstance = unsafe { &**emitter };
            let emitter_data: &EmitterData = emitter.get_emitter_data();
            let material_instance_data: &MaterialInstanceData = &emitter_data._material_instance_data.borrow();
            let pipeline_binding_data: &PipelineBindingData = material_instance_data.get_pipeline_binding_data(render_pass_pipeline_data_name);
            let render_pass_data: &RenderPassData = &pipeline_binding_data.get_render_pass_data().borrow();
            let pipeline_data: &PipelineData = &pipeline_binding_data.get_pipeline_data().borrow();

            if prev_pipeline_data != pipeline_data {
                if false == prev_pipeline_data.is_null() {
                    renderer_data.end_render_pass(command_buffer);
                }
                renderer_data.begin_render_pass_pipeline(command_buffer, swapchain_index, render_pass_data, pipeline_data, None);
                prev_pipeline_data = pipeline_data;
            }

            if prev_pipeline_binding_data != pipeline_binding_data {
                prev_pipeline_binding_data = pipeline_binding_data;
                renderer_data.bind_descriptor_sets(command_buffer, swapchain_index, &*pipeline_binding_data, None);
            }

            renderer_data.upload_push_constant_data(
                command_buffer,
                pipeline_data,
                &PushConstant_RenderParticle {
                    _allocated_emitter_index: emitter._allocated_emitter_index,
                    _allocated_particle_offset: emitter._allocated_particle_offset,
                    _reserved1: 0,
                    _reserved2: 0,
                }
            );
            renderer_data.draw_elements_instanced(command_buffer, &quad_geometry_data, &[], emitter._allocated_particle_count as u32);
        }
        renderer_data.end_render_pass(command_buffer);
    }
}