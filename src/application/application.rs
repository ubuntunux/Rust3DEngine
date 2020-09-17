use std::sync::Arc;

use vulkano::device::{Device, DeviceExtensions};
use vulkano::image::ImageUsage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::swapchain;
use vulkano::swapchain::{
    AcquireError, ColorSpace, FullscreenExclusive, PresentMode, SurfaceTransform, Swapchain,
    SwapchainCreationError,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};

use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use cgmath::Matrix4;
use cgmath::SquareMatrix;
use cgmath::Vector3;

use crate::frame::*;
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct TimeData
    { _acc_frame_time: f64
    , _acc_frame_count: i32
    , _average_frame_time: f64
    , _average_fps: f64
    , _current_time: f64
    , _elapsed_time: f64
    , _delta_time: f64
    }

impl Default for TimeData {
    fn default() -> TimeData {
        TimeData
            { _acc_frame_time: 0.0
            , _acc_frame_count: 0
            , _average_frame_time: 0.0
            , _average_fps: 0.0
            , _current_time: 0.0
            , _elapsed_time: 0.0
            , _delta_time: 0.0
            }
    }
}

#[derive(Debug, Clone)]
pub struct ApplicationData
    { pub _window: bool
    , _window_size_changed: bool
    , _window_size: (i32, i32)
    , _time_data: TimeData
    , _camera_move_speed: f32
    , _keyboard_input_data: bool // KeyboardInputData
    , _mouse_move_data: bool // MouseMoveData
    , _mouse_input_data: bool // MouseInputData
    , _scene_manager_data: bool // SceneManager.SceneManagerData
    , _renderer_data: bool // Renderer.RendererData
    , _resources: bool // Resource.Resources
    }

impl Default for ApplicationData {
    fn default() -> ApplicationData {
        ApplicationData
            { _window: false
            , _window_size_changed: false
            , _window_size: (1024, 768)
            , _time_data: TimeData::default()
            , _camera_move_speed: 1.0
            , _keyboard_input_data: false
            , _mouse_move_data: false
            , _mouse_input_data: false
            , _scene_manager_data: false
            , _renderer_data: false
            , _resources: false
            }
    }
}

impl ApplicationData {
    pub fn new() -> Arc<ApplicationData> {
        let app = ApplicationData::default();
        Arc::new(app)
    }
}

pub fn run_application() {
    let mut app = ApplicationData::new();
    //let mut qq = Arc::get_mut(&mut app).unwrap();

    let required_extensions = vulkano_win::required_extensions();
    let instance = Instance::new(None, &required_extensions, VULKAN_LAYERS.to_vec()).unwrap();
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();

    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
        .expect("couldn't find a graphical queue family");

    let device_ext = DeviceExtensions {
        khr_swapchain: KHR_SWAPCHAIN,
        ext_debug_utils: EXT_DEBUG_UTILS,
        ..DeviceExtensions::none()
    };
    let (device, mut queues) = Device::new(
        physical,
        physical.supported_features(),
        &device_ext,
        [(queue_family, 0.5)].iter().cloned(),
    )
        .unwrap();
    let queue = queues.next().unwrap();

    let (mut swapchain, mut images) = {
        let caps = surface.capabilities(physical).unwrap();
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let dimensions: [u32; 2] = surface.window().inner_size().into();

        Swapchain::new(
            device.clone(),
            surface.clone(),
            caps.min_image_count,
            format,
            dimensions,
            1,
            ImageUsage::color_attachment(),
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            FullscreenExclusive::Default,
            true,
            ColorSpace::SrgbNonLinear,
        )
            .unwrap()
    };

    // Here is the basic initialization for the deferred system.
    let mut frame_system = FrameSystem::new(queue.clone(), swapchain.format());
    let triangle_draw_system =
        TriangleDrawSystem::new(queue.clone(), frame_system.deferred_subpass());

    let mut recreate_swapchain = false;
    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            recreate_swapchain = true;
        }
        Event::RedrawEventsCleared => {
            previous_frame_end.as_mut().unwrap().cleanup_finished();

            if recreate_swapchain {
                let dimensions: [u32; 2] = surface.window().inner_size().into();
                let (new_swapchain, new_images) =
                    match swapchain.recreate_with_dimensions(dimensions) {
                        Ok(r) => r,
                        Err(SwapchainCreationError::UnsupportedDimensions) => return,
                        Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                    };

                swapchain = new_swapchain;
                images = new_images;
                recreate_swapchain = false;
            }

            let (image_num, suboptimal, acquire_future) =
                match swapchain::acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("Failed to acquire next image: {:?}", e),
                };

            if suboptimal {
                recreate_swapchain = true;
            }

            let future = previous_frame_end.take().unwrap().join(acquire_future);
            let mut frame =
                frame_system.frame(future, images[image_num].clone(), Matrix4::identity());
            let mut after_future = None;
            while let Some(pass) = frame.next_pass() {
                match pass {
                    Pass::Deferred(mut draw_pass) => {
                        let cb = triangle_draw_system.draw(draw_pass.viewport_dimensions());
                        draw_pass.execute(cb);
                    }
                    Pass::Lighting(mut lighting) => {
                        lighting.ambient_light([0.1, 0.1, 0.1]);
                        lighting.directional_light(Vector3::new(0.2, -0.1, -0.7), [0.6, 0.6, 0.6]);
                        lighting.point_light(Vector3::new(0.5, -0.5, -0.1), [1.0, 0.0, 0.0]);
                        lighting.point_light(Vector3::new(-0.9, 0.2, -0.15), [0.0, 1.0, 0.0]);
                        lighting.point_light(Vector3::new(0.0, 0.5, -0.05), [0.0, 0.0, 1.0]);
                    }
                    Pass::Finished(af) => {
                        after_future = Some(af);
                    }
                }
            }

            let future = after_future
                .unwrap()
                .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
                .then_signal_fence_and_flush();

            match future {
                Ok(future) => {
                    previous_frame_end = Some(future.boxed());
                }
                Err(FlushError::OutOfDate) => {
                    recreate_swapchain = true;
                    previous_frame_end = Some(sync::now(device.clone()).boxed());
                }
                Err(e) => {
                    println!("Failed to flush future: {:?}", e);
                    previous_frame_end = Some(sync::now(device.clone()).boxed());
                }
            }
        }
        _ => (),
    });
}
