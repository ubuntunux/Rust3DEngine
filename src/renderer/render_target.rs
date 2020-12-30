use ash::{
    vk,
};

use crate::constants;
use crate::renderer::renderer::RendererData;
use crate::renderer::fft_ocean;
use crate::vulkan_context::texture::{ TextureCreateInfo };
use log::Level::Debug;

#[repr(i32)]
#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum RenderTargetType {
    SceneColor,
    SceneColorCopy,
    SceneDepth,
    HierarchicalMinZ,
    BackBuffer,
    BackBufferCopy,
    SceneAlbedo,
    SceneNormal,
    SceneMaterial,
    SceneVelocity,
    TAAResolve,
    Bloom0,
    BloomTemp0,
    LightShaft,
    SSAO,
    SSAOTemp,
    Shadow,
    SSR,
    SSRResolved,
    SSRResolvedPrev,
    FFT_A,
    FFT_B,
    FFT_SLOPE_VARIANCE,
    MaxBound,
}

impl std::fmt::Display for RenderTargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for RenderTargetType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SceneColor" => Ok(RenderTargetType::SceneColor),
            "SceneColorCopy" => Ok(RenderTargetType::SceneColorCopy),
            "SceneDepth" => Ok(RenderTargetType::SceneDepth),
            "HierarchicalMinZ" => Ok(RenderTargetType::HierarchicalMinZ),
            "BackBuffer" => Ok(RenderTargetType::BackBuffer),
            "BackBufferCopy" => Ok(RenderTargetType::BackBufferCopy),
            "SceneAlbedo" => Ok(RenderTargetType::SceneAlbedo),
            "SceneNormal" => Ok(RenderTargetType::SceneNormal),
            "SceneMaterial" => Ok(RenderTargetType::SceneMaterial),
            "SceneVelocity" => Ok(RenderTargetType::SceneVelocity),
            "TAAResolve" => Ok(RenderTargetType::TAAResolve),
            "Bloom0" => Ok(RenderTargetType::Bloom0),
            "BloomTemp0" => Ok(RenderTargetType::BloomTemp0),
            "LightShaft" => Ok(RenderTargetType::LightShaft),
            "SSAO" => Ok(RenderTargetType::SSAO),
            "SSAOTemp" => Ok(RenderTargetType::SSAOTemp),
            "Shadow" => Ok(RenderTargetType::Shadow),
            "SSR" => Ok(RenderTargetType::SSR),
            "SSRResolved" => Ok(RenderTargetType::SSRResolved),
            "SSRResolvedPrev" => Ok(RenderTargetType::SSRResolvedPrev),
            "FFT_A" => Ok(RenderTargetType::FFT_A),
            "FFT_B" => Ok(RenderTargetType::FFT_B),
            "FFT_SLOPE_VARIANCE" => Ok(RenderTargetType::FFT_SLOPE_VARIANCE),
            _ => Err(format!("'{}' is not a valid value for RenderTargetType", s)),
        }
    }
}

pub fn get_render_target_create_infos(renderer_data: &RendererData) -> Vec<TextureCreateInfo<u8>> {
    let swapchain_data = &renderer_data._swapchain_data;
    let window_width = swapchain_data._swapchain_extent.width;
    let window_height = swapchain_data._swapchain_extent.height;
    let samples = vk::SampleCountFlags::TYPE_1;
    //let samples = min(vk::SampleCountFlags::TYPE_4, renderer_data._render_features._msaa_samples);
    let _enable_anisotropy = true;
    let disable_anisotropy = false;
    let hdr_texture_create_info = TextureCreateInfo {
        _texture_format: vk::Format::R16G16B16A16_SFLOAT,
        _texture_wrap_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
        _enable_anisotropy: disable_anisotropy,
        ..Default::default()
    };
    let texture_create_infos = vec![
        TextureCreateInfo {
            _texture_name: RenderTargetType::SceneColor.to_string(),
            _texture_width: window_width,
            _texture_height: window_height,
            _enable_mipmap: true,
            ..hdr_texture_create_info.clone()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::SceneColorCopy.to_string(),
            _texture_width: window_width,
            _texture_height: window_height,
            ..hdr_texture_create_info.clone()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::SceneDepth.to_string(),
            _texture_width: window_width,
            _texture_height: window_height,
            _texture_format: vk::Format::D32_SFLOAT,
            _texture_samples: samples,
            _texture_min_filter: vk::Filter::NEAREST,
            _texture_mag_filter: vk::Filter::NEAREST,
            _texture_wrap_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::HierarchicalMinZ.to_string(),
            _texture_width: window_width,
            _texture_height: window_height,
            _texture_format: vk::Format::R32_SFLOAT,
            _texture_samples: samples,
            _texture_min_filter: vk::Filter::NEAREST,
            _texture_mag_filter: vk::Filter::NEAREST,
            _texture_wrap_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            _enable_mipmap: true,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::BackBuffer.to_string(),
            _texture_width: window_width,
            _texture_height: window_height,
            _texture_format: vk::Format::R8G8B8A8_UNORM,
            _texture_samples: samples,
            _texture_wrap_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::BackBufferCopy.to_string(),
            _texture_width: window_width,
            _texture_height: window_height,
            _texture_format: vk::Format::R8G8B8A8_UNORM,
            _texture_samples: samples,
            _texture_wrap_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::SceneAlbedo.to_string(),
            _texture_width: window_width,
            _texture_height: window_height,
            _texture_format: vk::Format::R8G8B8A8_UNORM,
            _texture_wrap_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::SceneMaterial.to_string(),
            _texture_width: window_width,
            _texture_height: window_height,
            _texture_format: vk::Format::R8G8B8A8_UNORM,
            _texture_min_filter: vk::Filter::NEAREST,
            _texture_mag_filter: vk::Filter::NEAREST,
            _texture_wrap_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::SceneNormal.to_string(),
            _texture_width: window_width,
            _texture_height: window_height,
            _texture_format: vk::Format::R8G8B8A8_UNORM,
            _texture_min_filter: vk::Filter::NEAREST,
            _texture_mag_filter: vk::Filter::NEAREST,
            _texture_wrap_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::SceneVelocity.to_string(),
            _texture_width: window_width,
            _texture_height: window_height,
            _texture_format: vk::Format::R16G16_SFLOAT,
            _texture_min_filter: vk::Filter::NEAREST,
            _texture_mag_filter: vk::Filter::NEAREST,
            _texture_wrap_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::TAAResolve.to_string(),
            _texture_width: window_width,
            _texture_height: window_height,
            ..hdr_texture_create_info.clone()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::Bloom0.to_string(),
            _texture_width: window_width / 2,
            _texture_height: window_height / 2,
            _enable_mipmap: true,
            ..hdr_texture_create_info.clone()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::BloomTemp0.to_string(),
            _texture_width: window_width / 2,
            _texture_height: window_height / 2,
            _enable_mipmap: true,
            ..hdr_texture_create_info.clone()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::LightShaft.to_string(),
            _texture_width: window_width / 2,
            _texture_height: window_height / 2,
            _texture_format: vk::Format::R16G16B16A16_SFLOAT,
            _texture_wrap_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::SSAO.to_string(),
            _texture_width: window_width / 2,
            _texture_height: window_height / 2,
            _texture_format: vk::Format::R16_SFLOAT,
            _texture_wrap_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::SSAOTemp.to_string(),
            _texture_width: window_width / 2,
            _texture_height: window_height / 2,
            _texture_format: vk::Format::R16_SFLOAT,
            _texture_wrap_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::Shadow.to_string(),
            _texture_width: constants::SHADOW_MAP_SIZE,
            _texture_height: constants::SHADOW_MAP_SIZE,
            _texture_format: vk::Format::D32_SFLOAT,
            _texture_min_filter: vk::Filter::NEAREST,
            _texture_mag_filter: vk::Filter::NEAREST,
            _texture_wrap_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::SSR.to_string(),
            _texture_width: window_width / 4,
            _texture_height: window_height / 4,
            ..hdr_texture_create_info.clone()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::SSRResolved.to_string(),
            _texture_width: window_width / 4,
            _texture_height: window_height / 4,
            ..hdr_texture_create_info.clone()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::SSRResolvedPrev.to_string(),
            _texture_width: window_width / 4,
            _texture_height: window_height / 4,
            ..hdr_texture_create_info.clone()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::FFT_A.to_string(),
            _texture_width: fft_ocean::FFT_SIZE,
            _texture_height: fft_ocean::FFT_SIZE,
            _texture_layer: fft_ocean::FFT_LAYER_COUNT,
            _texture_view_type: vk::ImageViewType::TYPE_2D_ARRAY,
            _texture_format: vk::Format::R16G16B16A16_SFLOAT,
            _texture_min_filter: vk::Filter::LINEAR,
            _texture_mag_filter: vk::Filter::LINEAR,
            _texture_wrap_mode: vk::SamplerAddressMode::REPEAT,
            _enable_mipmap: true,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::FFT_B.to_string(),
            _texture_width: fft_ocean::FFT_SIZE,
            _texture_height: fft_ocean::FFT_SIZE,
            _texture_layer: fft_ocean::FFT_LAYER_COUNT,
            _texture_view_type: vk::ImageViewType::TYPE_2D_ARRAY,
            _texture_format: vk::Format::R16G16B16A16_SFLOAT,
            _texture_min_filter: vk::Filter::LINEAR,
            _texture_mag_filter: vk::Filter::LINEAR,
            _texture_wrap_mode: vk::SamplerAddressMode::REPEAT,
            _enable_mipmap: true,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        TextureCreateInfo {
            _texture_name: RenderTargetType::FFT_SLOPE_VARIANCE.to_string(),
            _texture_width: fft_ocean::N_SLOPE_VARIANCE,
            _texture_height: fft_ocean::N_SLOPE_VARIANCE,
            _texture_depth: fft_ocean::N_SLOPE_VARIANCE,
            _texture_view_type: vk::ImageViewType::TYPE_3D,
            _texture_format: vk::Format::R16G16B16A16_SFLOAT,
            _texture_min_filter: vk::Filter::LINEAR,
            _texture_mag_filter: vk::Filter::LINEAR,
            _texture_wrap_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            _enable_anisotropy: disable_anisotropy,
            ..Default::default()
        },
        // texture_slope_variance = CreateTexture(
        //     name='fft_ocean.slope_variance',
        //     texture_type=Texture3D,
        //     image_mode='RGBA',
        //     width=N_SLOPE_VARIANCE,
        //     height=N_SLOPE_VARIANCE,
        //     depth=N_SLOPE_VARIANCE,
        //     internal_format=GL_RGBA16F,
        //     texture_format=GL_RGBA,
        //     min_filter=GL_LINEAR,
        //     mag_filter=GL_LINEAR,
        //     wrap=GL_CLAMP_TO_EDGE,
        //     data_type=GL_FLOAT,
        // )
    ];
    texture_create_infos
}
