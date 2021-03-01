use std;
use ash::vk;

pub const ENGINE_NAME: &str = "RustEngine3D";
pub const ENGINE_VERSION: u32 = vk::make_version(1, 0, 0);
pub const DEPTH_FOMATS: [vk::Format; 5] = [
    vk::Format::D32_SFLOAT,
    vk::Format::D32_SFLOAT_S8_UINT,
    vk::Format::D24_UNORM_S8_UINT,
    vk::Format::D16_UNORM_S8_UINT,
    vk::Format::D16_UNORM
];
pub const DEPTH_STENCIL_FORMATS: [vk::Format; 3] = [
    vk::Format::D32_SFLOAT_S8_UINT,
    vk::Format::D24_UNORM_S8_UINT,
    vk::Format::D16_UNORM_S8_UINT
];
pub const CUBE_LAYER_COUNT: usize = 6;
pub const CUBE_TEXTURE_FACES: [&str; CUBE_LAYER_COUNT] = ["right", "left", "top", "bottom", "front", "back"];
pub const INVALID_QUEUE_INDEX: u32 = std::u32::MAX;
pub const WHOLE_LAYERS: u32 = std::u32::MAX;
pub const WHOLE_MIP_LEVELS: u32 = std::u32::MAX;
pub const SWAPCHAIN_IMAGE_COUNT: usize = 3;
pub const SWAPCHAIN_IMAGE_INDICES: [usize; SWAPCHAIN_IMAGE_COUNT] = [0, 1, 2];
pub const SWAPCHAIN_SURFACE_FORMATS: [vk::SurfaceFormatKHR; 2] = [
    vk::SurfaceFormatKHR { format: vk::Format::R8G8B8A8_SRGB, color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR },
    vk::SurfaceFormatKHR { format: vk::Format::B8G8R8A8_SRGB, color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR },
];
pub const MAX_FRAME_COUNT: usize = 2;
pub const FRAME_INDICES: [usize; MAX_FRAME_COUNT] = [0, 1];

#[derive(Debug, Clone)]
pub struct Constants {
    pub _vulkan_api_version: u32,
    pub _debug_message_level: vk::DebugUtilsMessageSeverityFlagsEXT,
    pub _vulkan_layers: Vec<String>,
    pub _require_device_extensions: Vec<String>,
    pub _max_descriptor_pool_alloc_count: usize,
    pub _enable_immediate_mode: bool,
    pub _enable_validation_layer: bool,
    pub _is_concurrent_mode: bool,
    pub _meter_per_unit: f32,
    pub _near: f32,
    pub _far: f32,
    pub _fov: f32,
    pub _camera_move_speed: f32,
    pub _camera_pan_speed: f32,
    pub _camera_rotation_speed: f32,
}

impl Default for Constants {
    fn default() -> Constants {
        #[cfg(target_os = "android")]
        let target_os_is_android: bool = true;
        #[cfg(not(target_os = "android"))]
        let target_os_is_android: bool = false;

        let enable_immediate_mode: bool;
        let enable_validation_layer: bool;
        let is_concurrent_mode: bool;

        if target_os_is_android {
            enable_immediate_mode = false;
            enable_validation_layer = false;
            is_concurrent_mode = false;
        } else {
            enable_immediate_mode = true;
            enable_validation_layer = true;
            is_concurrent_mode = true;
        }

        Constants {
            _vulkan_api_version: vk::make_version(1, 2, 0),
            _debug_message_level: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
            _vulkan_layers: vec!["VK_LAYER_LUNARG_standard_validation".to_string()],
            _require_device_extensions: vec!["VK_KHR_swapchain".to_string()],
            _max_descriptor_pool_alloc_count: 512,
            _enable_immediate_mode: enable_immediate_mode,
            _enable_validation_layer: enable_validation_layer,
            _is_concurrent_mode: is_concurrent_mode,
            _meter_per_unit: 1.0,
            _near: 0.1,
            _far: 2000.0,
            _fov: 60.0,
            _camera_move_speed: 10.0,
            _camera_pan_speed: 0.05,
            _camera_rotation_speed: 0.005,
        }
    }
}

///

#[cfg(target_os = "android")]
pub const VULKAN_API_VERSION: u32 = vk::make_version(1, 0, 0);
#[cfg(not(target_os = "android"))]
pub const VULKAN_API_VERSION: u32 = vk::make_version(1, 2, 0);
pub const DEBUG_MESSAGE_LEVEL: vk::DebugUtilsMessageSeverityFlagsEXT = vk::DebugUtilsMessageSeverityFlagsEXT::WARNING;
pub const VULKAN_LAYERS: [&str; 1] = ["VK_LAYER_LUNARG_standard_validation"];
pub const REQUIRE_DEVICE_EXTENSIONS: [&str; 1] = ["VK_KHR_swapchain"];
pub const MAX_DESCRIPTOR_POOL_ALLOC_COUNT: usize = 512;
#[cfg(target_os = "android")]
pub const ENABLE_IMMEDIATE_MODE: bool = false;
#[cfg(not(target_os = "android"))]
pub const ENABLE_IMMEDIATE_MODE: bool = true;
#[cfg(target_os = "android")]
pub const ENABLE_VALIDATION_LAYER: bool = false;
#[cfg(not(target_os = "android"))]
pub const ENABLE_VALIDATION_LAYER: bool = true;
#[cfg(target_os = "android")]
pub const IS_CONCURRENT_MODE: bool = false;
#[cfg(not(target_os = "android"))]
pub const IS_CONCURRENT_MODE: bool = true;
pub const METER_PER_UNIT: f32 = 1.0;
pub const NEAR: f32 = 0.1;
pub const FAR: f32 = 2000.0;
pub const FOV: f32 = 60.0;
pub const CAMERA_MOVE_SPEED: f32 = 10.0;
pub const CAMERA_PAN_SPEED: f32 = 0.05;
pub const CAMERA_ROTATION_SPEED: f32 = 0.005;
pub const SHADOW_MAP_SIZE: u32 = 2048;
pub const SHADOW_SAMPLES: i32 = 4;
pub const SHADOW_EXP: f32 = 100.0;
pub const SHADOW_BIAS: f32 = 0.005;
pub const SHADOW_DISTANCE: f32 = 50.0;
pub const SHADOW_DEPTH: f32 = 50.0;
pub const SHADOW_UPDATE_DISTANCE: f32 = 10.0;
pub const CAPTURE_HEIGHT_MAP_SIZE: u32 = 256;
pub const CAPTURE_HEIGHT_MAP_DISTANCE: f32 = 100.0;
pub const CAPTURE_HEIGHT_MAP_DEPTH: f32 = 100.0;
pub const CAPTURE_HEIGHT_MAP_UPDATE_DISTANCE: f32 = 10.0;
// SSAO_KERNEL_SIZE must match with scene_constants.glsl
pub const SSAO_KERNEL_SIZE: usize = 64;
pub const SSAO_RADIUS: f32 = 2.0;
pub const SSAO_NOISE_DIM: i32 = 4;
// MAX_BONES must match with scene_constants.glsl
pub const MAX_BONES: usize = 128 * 128;
pub const PRECOMPUTED_ROOT_MATRIX: bool = true; // precompute bone animation matrix with ancestor bone matrices.
pub const PRECOMPUTED_COMBINE_INV_BIND_MATRIX: bool = PRECOMPUTED_ROOT_MATRIX && false; // combine animation matrix with inv_bind_matrix.
pub const LIGHT_PROBE_SIZE: u32 = 256;
pub const RENDER_OBJECT_FOR_LIGHT_PROBE: bool = false;
