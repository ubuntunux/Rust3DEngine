use std::cell::RefMut;
use std::time;
use log;

use nalgebra::{
    Vector3,
};
use winit::event::{
    Event,
    VirtualKeyCode,
    WindowEvent,
};
use winit::event_loop::{
    ControlFlow,
    EventLoop
};

use crate::application::{scene_manager, SceneManagerData};
use crate::application::input;
use crate::resource;
use crate::renderer;
use crate::utilities::system::{ self, RcRefCell };
use crate::renderer::{RendererData, CameraCreateInfo};

#[derive(Debug, Clone)]
pub struct TimeData {
    _acc_frame_time: f64,
    _acc_frame_count: i32,
    _average_frame_time: f64,
    _average_fps: f64,
    _current_time: f64,
    _elapsed_time_prev: f64,
    _elapsed_time: f64,
    _delta_time: f64
}

pub fn create_time_data(elapsed_time: f64) -> TimeData {
    TimeData {
        _acc_frame_time: 0.0,
        _acc_frame_count: 0,
        _average_frame_time: 0.0,
        _average_fps: 0.0,
        _elapsed_time_prev: elapsed_time,
        _current_time: elapsed_time,
        _elapsed_time: elapsed_time,
        _delta_time: 0.0
    }
}

impl TimeData {
    pub fn update_time_data(&mut self, time_instance: &time::Instant) {
        let current_time = time_instance.elapsed().as_secs_f64();
        let previous_time = self._current_time;
        let delta_time = current_time - previous_time;
        self._elapsed_time_prev = self._elapsed_time;
        let elapsed_time = self._elapsed_time + delta_time;
        let acc_frame_time = self._acc_frame_time + delta_time;
        let acc_frame_count = self._acc_frame_count + 1;
        if 1.0 < acc_frame_time {
            let average_frame_time = acc_frame_time / (acc_frame_count as f64) * 1000.0;
            let average_fps = 1000.0 / average_frame_time;
            log::info!("{:.2}fps / {:.3}ms", average_fps, average_frame_time);
            self._acc_frame_time = 0.0;
            self._acc_frame_count = 0;
            self._average_frame_time = average_frame_time;
            self._average_fps = average_fps;
        } else {
            self._acc_frame_time = acc_frame_time;
            self._acc_frame_count = acc_frame_count;
        }
        self._current_time = current_time;
        self._elapsed_time = elapsed_time;
        self._delta_time = delta_time;
    }
}

pub struct ApplicationData {
    _window: bool,
    _window_size_changed: bool,
    _window_size: (u32, u32),
    _time_data: TimeData,
    _camera_move_speed: f32,
    _keyboard_input_data: Box<input::KeyboardInputData>,
    _mouse_move_data: Box<input::MouseMoveData>,
    _mouse_input_data: Box<input::MouseInputData>,
    _scene_manager_data: RcRefCell<scene_manager::SceneManagerData>,
    _renderer_data: RcRefCell<renderer::RendererData>,
    _resources: RcRefCell<resource::Resources>
}

impl ApplicationData {
    pub fn terminate_applicateion(&mut self) {
        self._resources.borrow_mut().destroy_resources(&self._renderer_data.borrow_mut());
        self._renderer_data.borrow_mut().destroy_renderer_data();
    }
}


pub fn run_application(app_name: &str, app_version: u32, window_size: (u32, u32)) {
    log::info!("run_application");
    let time_instance = time::Instant::now();
    let elapsed_time = time_instance.elapsed().as_secs_f64();
    let event_loop = EventLoop::new();
    let (width, height) = window_size;
    let mouse_pos = (width / 2, height / 2);
    let resources = resource::create_resources();
    let renderer_data: RcRefCell<RendererData> = renderer::create_renderer_data(app_name, app_version, window_size, &event_loop, resources.clone());
    let scene_manager_data = scene_manager::create_scene_manager_data(renderer_data.clone(), resources.clone());
    let keyboard_input_data = input::create_keyboard_input_data();
    let mouse_move_data = input::create_mouse_move_data(mouse_pos);
    let mouse_input_data = input::create_mouse_input_data();
    let application_data = system::newRcRefCell(
        ApplicationData {
            _window: false,
            _window_size_changed: false,
            _window_size: window_size,
            _time_data: create_time_data(elapsed_time),
            _camera_move_speed: 1.0,
            _keyboard_input_data: keyboard_input_data.clone(),
            _mouse_move_data: mouse_move_data.clone(),
            _mouse_input_data: mouse_input_data.clone(),
            _scene_manager_data: scene_manager_data.clone(),
            _renderer_data: renderer_data.clone(),
            _resources: resources.clone(),
        }
    );


    resources.borrow_mut().initialize_resources(&renderer_data.borrow());

    let camera_data = CameraCreateInfo {
        aspect: if 0 != height { width as f32 / height as f32 } else { 1.0 },
        position: Vector3::new(0.0, 0.0, 10.0),
        ..Default::default()
    };

    scene_manager_data.borrow_mut().open_scene_manager_data(&camera_data);

    // main loop
    let mut render_scene: bool = false;
    let mut run_application: bool = true;
    event_loop.run(move |event, __window_target, control_flow|{
        let mut application_data: RefMut<ApplicationData> = application_data.borrow_mut();
        let mut renderer_data: RefMut<RendererData> = renderer_data.borrow_mut();
        let mut scene_manager_data: RefMut<SceneManagerData> = scene_manager_data.borrow_mut();

        if run_application {
            application_data._time_data.update_time_data(&time_instance);
            let elapsed_time = application_data._time_data._elapsed_time;
            let delta_time = application_data._time_data._delta_time;

            render_scene = false;
            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit;
                    application_data.terminate_applicateion();
                    run_application = false;
                    return;
                },
                Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } => {
                    if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                        *control_flow = ControlFlow::Exit;
                        application_data.terminate_applicateion();
                        run_application = false;
                        return;
                    }
                },
                Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                    renderer_data.set_need_recreate_swapchain(true);
                },
                Event::RedrawEventsCleared => {
                    render_scene = true;
                },
                _ => { },
            }

            if renderer_data.get_need_recreate_swapchain() {
                if false == renderer_data.get_is_first_resize_event() {
                    renderer_data.resize_window();
                    let window_size = renderer_data._window.inner_size();
                    let aspect: f32 = if 0 != window_size.height {
                        window_size.width as f32 / window_size.height as f32
                    } else {
                        1.0
                    };
                    scene_manager_data.get_main_camera().borrow_mut().set_aspect(aspect);
                }
                renderer_data.set_is_first_resize_event(false);
                renderer_data.set_need_recreate_swapchain(false);
            }

            scene_manager_data.update_scene_manager_data(elapsed_time, delta_time);

            if render_scene {
                renderer_data.render_scene(scene_manager_data, elapsed_time, delta_time);
            }
        }
    });
}
