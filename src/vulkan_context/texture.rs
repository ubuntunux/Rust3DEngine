use std::cmp::max;
use std::mem::align_of;
use std::os::raw::c_void;

use ash::{
    vk,
    Device,
    Instance,
};
use ash::util::Align;
use ash::version::{
    DeviceV1_0,
    InstanceV1_0
};

use crate::constants;
use crate::vulkan_context::buffer;
use crate::vulkan_context::vulkan_context::run_commands_once;

#[derive(Debug, Clone)]
pub struct TextureCreateInfo {
    pub _texture_width: u32,
    pub _texture_height: u32,
    pub _texture_depth: u32,
    pub _texture_format: vk::Format,
    pub _texture_view_type: vk::ImageViewType,
    pub _texture_samples: vk::SampleCountFlags,
    pub _texture_min_filter: vk::Filter,
    pub _texture_mag_filter: vk::Filter,
    pub _texture_wrap_mode: vk::SamplerAddressMode,
    pub _enable_mipmap: bool,
    pub _enable_anisotropy: bool,
    pub _immutable: bool,
    pub _texture_initial_datas: Vec<u8>
}

#[derive(Debug, Clone)]
pub struct TextureData {
    pub _texture_data_name: String,
    pub _image: vk::Image,
    pub _image_view: vk::ImageView,
    pub _image_memory: vk::DeviceMemory,
    pub _image_sampler:vk::Sampler,
    pub _image_format: vk::Format,
    pub _image_width: u32,
    pub _image_height: u32,
    pub _image_depth: u32,
    pub _image_mip_levels: u32,
    pub _image_sample_count: vk::SampleCountFlags,
    pub _descriptor_image_info: vk::DescriptorImageInfo,
}

#[derive(Clone, Debug, Copy)]
pub enum ImageLayoutTransition {
    TransferUndefToTransferDst,
    TransferDstToShaderReadOnly,
    TransferUndefToDepthStencilAttachemnt,
    TransferUndefToColorAttachemnt,
}

#[derive(Debug, Clone)]
pub struct TransitionDependent {
    pub _old_layout: vk::ImageLayout,
    pub _new_layout: vk::ImageLayout,
    pub _src_access_mask: vk::AccessFlags,
    pub _dst_access_mask: vk::AccessFlags,
    pub _src_stage_mask: vk::PipelineStageFlags,
    pub _dst_stage_mask: vk::PipelineStageFlags,
}

impl Default for TextureCreateInfo {
    fn default() -> TextureCreateInfo {
        TextureCreateInfo {
            _texture_width: 1,
            _texture_height: 1,
            _texture_depth: 1,
            _texture_format: vk::Format::R8G8B8A8_UNORM,
            _texture_view_type: vk::ImageViewType::TYPE_2D,
            _texture_samples: vk::SampleCountFlags::TYPE_1,
            _texture_min_filter: vk::Filter::LINEAR,
            _texture_mag_filter: vk::Filter::LINEAR,
            _texture_wrap_mode: vk::SamplerAddressMode::REPEAT,
            _enable_mipmap: true,
            _enable_anisotropy: true,
            _immutable: true,
            _texture_initial_datas: Vec::new(),
        }
    }
}

pub fn get_transition_dependent(image_layout_transition: ImageLayoutTransition) -> TransitionDependent {
    match image_layout_transition {
        ImageLayoutTransition::TransferUndefToTransferDst => TransitionDependent {
            _old_layout: vk::ImageLayout::UNDEFINED,
            _new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            _src_access_mask: vk::AccessFlags::empty(),
            _dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
            _src_stage_mask: vk::PipelineStageFlags::TOP_OF_PIPE,
            _dst_stage_mask: vk::PipelineStageFlags::TRANSFER,
        },
        ImageLayoutTransition::TransferDstToShaderReadOnly => TransitionDependent {
            _old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            _new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            _src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
            _dst_access_mask: vk::AccessFlags::SHADER_READ,
            _src_stage_mask: vk::PipelineStageFlags::TRANSFER,
            _dst_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
        },
        ImageLayoutTransition::TransferUndefToDepthStencilAttachemnt => TransitionDependent {
            _old_layout: vk::ImageLayout::UNDEFINED,
            _new_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            _src_access_mask: vk::AccessFlags::empty(),
            _dst_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            _src_stage_mask: vk::PipelineStageFlags::TOP_OF_PIPE,
            _dst_stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        },
        ImageLayoutTransition::TransferUndefToColorAttachemnt => TransitionDependent {
            _old_layout: vk::ImageLayout::UNDEFINED,
            _new_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            _src_access_mask: vk::AccessFlags::empty(),
            _dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            _src_stage_mask: vk::PipelineStageFlags::TOP_OF_PIPE,
            _dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        },
    }
}

pub fn image_view_type_to_image_type(image_view_type: vk::ImageViewType) -> vk::ImageType {
    match image_view_type {
        vk::ImageViewType::TYPE_1D => vk::ImageType::TYPE_1D,
        vk::ImageViewType::TYPE_2D => vk::ImageType::TYPE_2D,
        vk::ImageViewType::TYPE_2D_ARRAY => vk::ImageType::TYPE_2D,
        vk::ImageViewType::CUBE => vk::ImageType::TYPE_2D,
        vk::ImageViewType::CUBE_ARRAY => vk::ImageType::TYPE_2D,
        vk::ImageViewType::TYPE_3D => vk::ImageType::TYPE_3D,
        _ => vk::ImageType::TYPE_2D,
    }
}

pub fn next_mipmap_size(n: i32) -> i32 {
    if 1 < n { n/2 } else { 1 }
}

pub fn calc_mip_levels(image_width: u32, image_height: u32, image_depth: u32) -> u32 {
    let max_size: f32 = max(image_width, max(image_height, image_depth)) as f32;
    max_size.log2().floor() as u32 + 1
}

pub fn image_barrier_struct(
    image: vk::Image,
    aspect_mask: vk::ImageAspectFlags,
    mip_level: u32,
    layer_count: u32,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    src_access_mask: vk::AccessFlags,
    dst_access_mask: vk::AccessFlags,
) -> vk::ImageMemoryBarrier {
    vk::ImageMemoryBarrier {
        old_layout,
        new_layout,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask, // vk::ImageAspectFlags::COLOR,
            base_mip_level: mip_level,
            level_count: 1,
            base_array_layer: 0,
            layer_count,
            ..Default::default()
        },
        src_access_mask,
        dst_access_mask,
        ..Default::default()
    }
}

pub fn image_blit_struct(
    image_aspect_mask: vk::ImageAspectFlags,
    mip_level: u32,
    src_width: i32,
    src_height: i32,
    src_depth: i32,
    layer_count: u32
) -> vk::ImageBlit {
    vk::ImageBlit {
        src_offsets: [
            vk::Offset3D { x: 0, y: 0, z: 0 },
            vk::Offset3D { x: src_width, y: src_height, z: src_depth }
        ],
        dst_offsets: [
            vk::Offset3D { x: 0, y: 0, z: 0 },
            vk::Offset3D {
                x: next_mipmap_size(src_width),
                y: next_mipmap_size(src_height),
                z: next_mipmap_size(src_depth)
            }
        ],
        src_subresource: vk::ImageSubresourceLayers {
            aspect_mask: image_aspect_mask,
            mip_level: mip_level - 1,
            base_array_layer: 0,
            layer_count
        },
        dst_subresource: vk::ImageSubresourceLayers {
            aspect_mask: image_aspect_mask,
            mip_level,
            base_array_layer: 0,
            layer_count
        },
        ..Default::default()
    }
}

pub fn create_mipmap(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    aspect_mask: vk::ImageAspectFlags,
    mip_level: u32,
    src_width: i32,
    src_height: i32,
    src_depth: i32,
    layer_count: u32
) {
    unsafe {
        let barriers: [vk::ImageMemoryBarrier; 1] = [
            image_barrier_struct(
                image,
                aspect_mask,
                mip_level - 1,
                layer_count,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::TRANSFER_READ,
            )
        ];
        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &barriers
        );

        let image_blits = [
            image_blit_struct(
                aspect_mask,
                mip_level,
                src_width,
                src_height,
                src_depth,
                layer_count
            )
        ];
        device.cmd_blit_image(
            command_buffer,
            image,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &image_blits,
            vk::Filter::LINEAR
        );

        let barriers: [vk::ImageMemoryBarrier; 1] = [
            image_barrier_struct(
                image,
                aspect_mask,
                mip_level - 1,
                layer_count,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::AccessFlags::TRANSFER_READ,
                vk::AccessFlags::SHADER_READ,
            )
        ];
        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &barriers
        );
    }
}

pub fn generate_mipmaps(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    aspect_mask: vk::ImageAspectFlags,
    format: vk::Format,
    width: i32,
    height: i32,
    depth: i32,
    mip_levels: u32,
    layer_count: u32,
) {
    unsafe {
        let format_properties: vk::FormatProperties = instance.get_physical_device_format_properties(physical_device, format);
        let supported = format_properties.optimal_tiling_features & vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR;
        if supported == vk::FormatFeatureFlags::empty() {
            panic!("texture image format does not support linear blitting!");
        }

        let mut mip_width: i32 = width;
        let mut mip_height: i32 = height;
        let mut mip_depth: i32 = depth;
        for mip_level in 1..mip_levels {
            mip_width = next_mipmap_size(mip_width);
            mip_height = next_mipmap_size(mip_height);
            mip_depth = next_mipmap_size(mip_depth);
            create_mipmap(
                device,
                command_buffer,
                image,
                aspect_mask,
                mip_level,
                mip_width,
                mip_height,
                mip_depth,
                layer_count
            );
        }

        let barriers: [vk::ImageMemoryBarrier; 1] = [
            image_barrier_struct(
                image,
                aspect_mask,
                mip_levels - 1,
                layer_count,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
            )
        ];
        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &barriers
        );
    }
}

pub fn find_supported_format(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    require_format: vk::Format,
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> vk::Format {
    unsafe {
        let candidates: Vec<vk::Format> = constants::DEPTH_FOMATS.iter().filter(|__format| {
            let format_properties = instance.get_physical_device_format_properties(physical_device, require_format);
            match tiling {
                vk::ImageTiling::LINEAR => (format_properties.linear_tiling_features & features) == features,
                vk::ImageTiling::OPTIMAL => (format_properties.optimal_tiling_features & features) == features,
                _ => false
            }
        }).map(|format| *format).collect();

        if 0 < candidates.len() {
            return if candidates.contains(&require_format) {
                require_format
            } else {
                candidates[0]
            }
        }
    }
    panic!("failed to find supported format");
}

pub fn create_image_sampler(
    device: &Device,
    mip_levels: u32,
    min_filter: vk::Filter,
    mag_filter: vk::Filter,
    sampler_address_mode: vk::SamplerAddressMode,
    anisotropy_enable: vk::Bool32
) -> vk::Sampler {
    let sampler_create_info = vk::SamplerCreateInfo {
        min_filter,
        mag_filter,
        address_mode_u: sampler_address_mode,
        address_mode_v: sampler_address_mode,
        address_mode_w: sampler_address_mode,
        anisotropy_enable,
        max_anisotropy: 16.0,
        border_color: vk::BorderColor::INT_OPAQUE_BLACK,
        unnormalized_coordinates: vk::FALSE,
        compare_enable: vk::FALSE,
        compare_op: vk::CompareOp::NEVER,
        mipmap_mode: vk::SamplerMipmapMode::LINEAR,
        mip_lod_bias: 0.0,
        min_lod: 0.0,
        max_lod: mip_levels as f32,
        ..Default::default()
    };

    unsafe {
        let sampler = device.create_sampler(&sampler_create_info, None).expect("Failed to create sampler.");
        log::info!("create_image_sampler: {:?}", sampler);
        sampler
    }
}

pub fn destroy_image_sampler(device: &Device, sampler: vk::Sampler) {
    unsafe {
        log::info!("destroy_image_sampler: {:?}", sampler);
        device.destroy_sampler(sampler, None);
    }
}

pub fn create_image_view(
    device: &Device,
    image: vk::Image,
    view_type:vk::ImageViewType,
    format: vk::Format,
    aspect_flags: vk::ImageAspectFlags,
    layer_count: u32,
    mip_levels: u32
) -> vk::ImageView {
    let create_view_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(view_type)
        .format(format)
        .components(vk::ComponentMapping {
            r: vk::ComponentSwizzle::IDENTITY,
            g: vk::ComponentSwizzle::IDENTITY,
            b: vk::ComponentSwizzle::IDENTITY,
            a: vk::ComponentSwizzle::IDENTITY,
        })
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: aspect_flags,
            base_mip_level: 0,
            level_count: layer_count,
            base_array_layer: 0,
            layer_count: mip_levels,
        })
        .build();
    unsafe {
        device.create_image_view(&create_view_info, None).expect("vkCreateImageView failed!")
    }
}

pub fn destroy_image_view(device: &Device, image_view: vk::ImageView) {
    unsafe {
        device.destroy_image_view(image_view, None);
    }
}

pub fn transition_image_layout(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    format: vk::Format,
    transition: ImageLayoutTransition,
    layer_count: u32,
    mip_levels: u32
) {
    let transition_dependent = get_transition_dependent(transition);
    let aspect_mask = match transition_dependent._new_layout {
        vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL =>
            if constants::DEPTH_STENCIL_FORMATS.contains(&format) {
                vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
            } else {
                vk::ImageAspectFlags::DEPTH
            },
        _ => vk::ImageAspectFlags::COLOR
    };
    let barriers: [vk::ImageMemoryBarrier; 1] = [
        vk::ImageMemoryBarrier {
            old_layout: transition_dependent._old_layout,
            new_layout: transition_dependent._new_layout,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: mip_levels,
                base_array_layer: 0,
                layer_count,
                ..Default::default()
            },
            src_access_mask: transition_dependent._src_access_mask,
            dst_access_mask: transition_dependent._dst_access_mask,
            ..Default::default()
        }
    ];
    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            transition_dependent._src_stage_mask,
            transition_dependent._dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &barriers);
    }
}

pub fn create_image(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
    image_type: vk::ImageType,
    width: u32,
    height: u32,
    depth: u32,
    layer_count: u32,
    mip_levels: u32,
    samples: vk::SampleCountFlags,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    image_create_flags: vk::ImageCreateFlags,
    memory_property_flags: vk::MemoryPropertyFlags
) -> (vk::DeviceMemory, vk::Image) {
    unsafe {
        let image_create_info = vk::ImageCreateInfo {
            flags: image_create_flags,
            image_type,
            extent: vk::Extent3D { width, height, depth, },
            mip_levels,
            array_layers: layer_count,
            format,
            tiling,
            initial_layout: vk::ImageLayout::UNDEFINED,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            samples,
            ..Default::default()
        };

        instance.get_physical_device_image_format_properties(
            physical_device,
            format,
            image_type,
            tiling,
            usage,
            image_create_flags
        ).expect("vkGetPhysicalDeviceImageFormatProperties failed!");

        let image = device.create_image(&image_create_info, None).expect("vkCreateImage failed!");
        let memory_requirements = device.get_image_memory_requirements(image);
        let memory_type_index = buffer::find_memory_type_index(&memory_requirements, memory_properties, memory_property_flags).unwrap();
        let image_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: memory_requirements.size,
            memory_type_index,
            ..Default::default()
        };
        let image_memory = device.allocate_memory(&image_allocate_info, None).expect("vkAllocateMemory failed!");
        device.bind_image_memory(image, image_memory, 0).expect("vkBindImageMemory failed!");
        (image_memory, image)
    }
}

pub fn destroy_image(device: &Device, image: vk::Image, image_memory: vk::DeviceMemory) {
    unsafe {
        device.destroy_image(image, None);
        device.free_memory(image_memory, None);
    }
}

pub fn copy_buffer_to_image(
    device: &Device,
    command_pool: vk::CommandPool,
    command_queue: vk::Queue,
    buffer: vk::Buffer,
    image: vk::Image,
    image_aspect: vk::ImageAspectFlags,
    width: u32,
    height: u32,
    depth: u32,
    layer_count: u32,
) {
    let regions: [vk::BufferImageCopy; 1] = [
        vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: image_aspect,
                mip_level: 0,
                base_array_layer: 0,
                layer_count,
            },
            image_offset: vk::Offset3D { x: 0, y: 0 , z: 0 },
            image_extent:  vk::Extent3D { width, height, depth },
            ..Default::default()
        }
    ];
    run_commands_once(device, command_pool, command_queue, |device: &Device, command_buffer: vk::CommandBuffer| {
            unsafe {
                device.cmd_copy_buffer_to_image(command_buffer, buffer, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &regions);
            }
        }
    );
}

pub fn create_render_target(
    texture_data_name: &String,
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
    command_pool: vk::CommandPool,
    command_queue: vk::Queue,
    texture_create_info: &TextureCreateInfo,
) -> TextureData {
    let enable_anisotropy = match texture_create_info._enable_anisotropy {
        true => vk::TRUE,
        _ => vk::FALSE
    };
    let (texture_create_flags, layer_count) = match texture_create_info._texture_view_type {
        vk::ImageViewType::CUBE => (vk::ImageCreateFlags::CUBE_COMPATIBLE, 6),
        _ => (vk::ImageCreateFlags::empty(), 1),
    };
    let mip_levels = match texture_create_info._enable_mipmap {
        true => calc_mip_levels(texture_create_info._texture_width, texture_create_info._texture_height, texture_create_info._texture_depth),
        _ => 1
    };
    let is_depth_format = constants::DEPTH_FOMATS.contains(&texture_create_info._texture_format);
    let common_usage = vk::ImageUsageFlags::INPUT_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::SAMPLED;
    let image_type = image_view_type_to_image_type(texture_create_info._texture_view_type);
    let (image_usage, image_aspect, image_layout_transition, image_format) =
        if is_depth_format {
            let depth_format = find_supported_format(
                instance,
                physical_device,
                texture_create_info._texture_format,
                vk::ImageTiling::OPTIMAL,
                vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
            );
            (
                common_usage | vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                vk::ImageAspectFlags::DEPTH,
                ImageLayoutTransition::TransferUndefToDepthStencilAttachemnt,
                depth_format
            )
        } else {
            (
                common_usage | vk::ImageUsageFlags::COLOR_ATTACHMENT,
                vk::ImageAspectFlags::COLOR,
                ImageLayoutTransition::TransferUndefToColorAttachemnt,
                texture_create_info._texture_format
            )
        };
    let (image_memory, image) = create_image(
        instance,
        device,
        physical_device,
        memory_properties,
        image_type,
        texture_create_info._texture_width,
        texture_create_info._texture_height,
        texture_create_info._texture_depth,
        layer_count,
        mip_levels,
        texture_create_info._texture_samples,
        image_format,
        vk::ImageTiling::OPTIMAL,
        image_usage,
        texture_create_flags,
        vk::MemoryPropertyFlags::DEVICE_LOCAL
    );
    run_commands_once(device, command_pool, command_queue, |device: &Device, command_buffer: vk::CommandBuffer| {
            transition_image_layout(device, command_buffer, image, image_format, image_layout_transition, layer_count, mip_levels);
        }
    );
    let image_view = create_image_view(
        device,
        image,
        texture_create_info._texture_view_type,
        image_format,
        image_aspect,
        layer_count,
        mip_levels
    );
    let image_sampler = create_image_sampler(
        device,
        mip_levels,
        texture_create_info._texture_min_filter,
        texture_create_info._texture_mag_filter,
        texture_create_info._texture_wrap_mode,
        enable_anisotropy
    );
    let descriptor_image_info = vk::DescriptorImageInfo {
        sampler: image_sampler,
        image_view,
        image_layout: vk::ImageLayout::GENERAL
    };
    let texture_data = TextureData {
        _texture_data_name: texture_data_name.clone(),
        _image: image,
        _image_view: image_view,
        _image_memory: image_memory,
        _image_sampler: image_sampler,
        _image_format: image_format,
        _image_width: texture_create_info._texture_width,
        _image_height: texture_create_info._texture_height,
        _image_depth: texture_create_info._texture_depth,
        _image_mip_levels: mip_levels,
        _image_sample_count: texture_create_info._texture_samples,
        _descriptor_image_info: descriptor_image_info
    };
    log::info!("create_render_target: {} {:?} {:?} {} {} {}",
             texture_data_name,
             texture_create_info._texture_view_type,
             image_format,
             texture_create_info._texture_width,
             texture_create_info._texture_height,
             texture_create_info._texture_depth
    );
    log::info!("    TextureData: image: {:?}, image_view: {:?}, image_memory: {:?}, sampler: {:?}", image, image_view, image_memory, image_sampler);
    texture_data
}

pub fn create_texture_data(
    texture_data_name: &String,
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
    command_pool: vk::CommandPool,
    command_queue: vk::Queue,
    texture_create_info: &TextureCreateInfo,
) -> TextureData {
    let image_datas = &texture_create_info._texture_initial_datas;
    let buffer_size = image_datas.len() as vk::DeviceSize;
    let enable_anisotropy = match texture_create_info._enable_anisotropy {
        true => vk::TRUE,
        _ => vk::FALSE
    };
    let (texture_create_flags, layer_count) = match texture_create_info._texture_view_type {
        vk::ImageViewType::CUBE => (vk::ImageCreateFlags::CUBE_COMPATIBLE, 6),
        _ => (vk::ImageCreateFlags::empty(), 1),
    };
    let mip_levels = match texture_create_info._enable_mipmap {
        true => calc_mip_levels(texture_create_info._texture_width, texture_create_info._texture_height, texture_create_info._texture_depth),
        _ => 1
    };
    let image_aspect = vk::ImageAspectFlags::COLOR;
    let image_type = image_view_type_to_image_type(texture_create_info._texture_view_type);
    let image_usage = vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED;
    // we don't need to access the vk::DeviceMemory of the image, copyBufferToImage works with the vk::Image
    let (image_memory, image) = create_image(
        instance,
        device,
        physical_device,
        memory_properties,
        image_type,
        texture_create_info._texture_width,
        texture_create_info._texture_height,
        texture_create_info._texture_depth,
        layer_count,
        mip_levels,
        texture_create_info._texture_samples,
        texture_create_info._texture_format,
        vk::ImageTiling::OPTIMAL,
        image_usage,
        texture_create_flags,
        vk::MemoryPropertyFlags::DEVICE_LOCAL
    );
    run_commands_once(device, command_pool, command_queue, |device: &Device, command_buffer: vk::CommandBuffer| {
        transition_image_layout(
            device,
            command_buffer,
            image,
            texture_create_info._texture_format,
            ImageLayoutTransition::TransferUndefToTransferDst,
            layer_count,
            mip_levels
        );
    });

    // create temporary staging buffer
    let staging_buffer_usage_flags = vk::BufferUsageFlags::TRANSFER_SRC;
    let staging_buffer_memory_property_flags = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    let staging_buffer_data: buffer::BufferData = buffer::create_buffer_data(
        device,
        memory_properties,
        buffer_size,
        staging_buffer_usage_flags,
        staging_buffer_memory_property_flags
    );

    unsafe {
        // upload data
        let stageing_buffer_ptr: *mut c_void = device.map_memory(
            staging_buffer_data._buffer_memory,
            0,
            buffer_size,
            vk::MemoryMapFlags::empty()
        ).expect("Failed to map_memory!");
        let mut stageing_buffer_slice = Align::new(
            stageing_buffer_ptr,
            align_of::<u32>() as u64,
            staging_buffer_data._buffer_memory_requirements.size,
        );
        stageing_buffer_slice.copy_from_slice(image_datas);
        device.unmap_memory(staging_buffer_data._buffer_memory);
    }

    copy_buffer_to_image(
        device,
        command_pool,
        command_queue,
        staging_buffer_data._buffer,
        image,
        image_aspect,
        texture_create_info._texture_width,
        texture_create_info._texture_height,
        texture_create_info._texture_depth,
        layer_count,
    );

    // generateMipmaps does this as a side effect:
    // transitionImageLayout image VK_FORMAT_R8G8B8A8_UNORM TransferDst_ShaderReadOnly mipLevels
    run_commands_once(device, command_pool, command_queue, |device: &Device, command_buffer: vk::CommandBuffer| {
        generate_mipmaps(
            instance,
            device,
            physical_device,
            command_buffer,
            image,
            image_aspect,
            texture_create_info._texture_format,
            texture_create_info._texture_width as i32,
            texture_create_info._texture_height as i32,
            texture_create_info._texture_depth as i32,
            mip_levels,
            layer_count,
        );
    });

    // destroy staging buffer
    buffer::destroy_buffer_data(device, &staging_buffer_data);

    // create texture datas
    let image_view = create_image_view(
        device,
        image,
        texture_create_info._texture_view_type,
        texture_create_info._texture_format,
        image_aspect,
        layer_count,
        mip_levels
    );
    let image_sampler = create_image_sampler(
        device,
        mip_levels,
        texture_create_info._texture_min_filter,
        texture_create_info._texture_mag_filter,
        texture_create_info._texture_wrap_mode,
        enable_anisotropy
    );
    let descriptor_image_info = vk::DescriptorImageInfo {
        sampler: image_sampler,
        image_view,
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
    };
    let texture_data = TextureData {
        _texture_data_name: texture_data_name.clone(),
        _image: image.clone(),
        _image_view: image_view.clone(),
        _image_memory: image_memory,
        _image_sampler: image_sampler,
        _image_format: texture_create_info._texture_format,
        _image_width: texture_create_info._texture_width,
        _image_height: texture_create_info._texture_height,
        _image_depth: texture_create_info._texture_depth,
        _image_mip_levels: mip_levels,
        _image_sample_count: texture_create_info._texture_samples,
        _descriptor_image_info: descriptor_image_info
    };

    log::info!("create_texture_data: {} {:?} {:?} {} {} {}",
             texture_data_name,
             texture_create_info._texture_view_type,
             texture_create_info._texture_format,
             texture_create_info._texture_width,
             texture_create_info._texture_height,
             texture_create_info._texture_depth
    );
    log::info!("    TextureData: image: {:?}, image_view: {:?}, image_memory: {:?}, sampler: {:?}", image, image_view, image_memory, image_sampler);
    texture_data
}

pub fn destroy_texture_data(device: &Device, texture_data: &TextureData) {
    unsafe {
        log::info!("destroy_texture_data({}): image: {:?}, image_view: {:?}, image_memory: {:?}, sampler: {:?}",
            texture_data._texture_data_name,
            texture_data._image,
            texture_data._image_view,
            texture_data._image_memory,
            texture_data._image_sampler
        );
        device.destroy_sampler(texture_data._image_sampler, None);
        device.destroy_image_view(texture_data._image_view, None);
        destroy_image(device, texture_data._image, texture_data._image_memory);
    }
}