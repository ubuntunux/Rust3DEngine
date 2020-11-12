use std::thread;
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
use winit_input_helper::WinitInputHelper;

use crate::constants;
use crate::application::{scene_manager, SceneManagerData};
use crate::application::input;
use crate::resource::{self, Resources};
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
    pub fn terminate_applicateion(
        &mut self,
        scene_manager_data: &mut SceneManagerData,
        resources: &mut Resources,
        renderer_data: &mut RendererData,
    ) {
        scene_manager_data.close_scene_manager_data();
        resources.destroy_resources(renderer_data);
        renderer_data.destroy_renderer_data();
    }

    pub fn update_event(&mut self, scene_manager_data: &SceneManagerData, input_helper: &WinitInputHelper) {
        // TODO: Use Queue or Stack for IO Events
        // let keyboard_input_data = self._keyboard_input_data.borrow();
        //let mouse_move_data: &mut input::MouseMoveData = &mut self._mouse_move_data;
        // let mouse_input_data = self._mouse_input_data.borrow_mut();

        let delta_time = self._time_data._delta_time;
        let (mouse_delta_x, mouse_delta_y)  = input_helper.mouse_diff();
        let mouse_pos = input_helper.mouse();
        const MOUSE_LEFT: usize = 0;
        const MOUSE_RIGHT: usize = 1;
        const MOUSE_MIDDLE: usize = 2;
        let btn_left: bool = input_helper.mouse_held(MOUSE_LEFT);
        let btn_right: bool = input_helper.mouse_held(MOUSE_RIGHT);
        let btn_middle: bool = input_helper.mouse_held(MOUSE_MIDDLE);

        let pressed_key_A = input_helper.key_pressed(VirtualKeyCode::A);
        let pressed_key_D = input_helper.key_pressed(VirtualKeyCode::D);
        let pressed_key_W = input_helper.key_pressed(VirtualKeyCode::W);
        let pressed_key_S = input_helper.key_pressed(VirtualKeyCode::S);
        let pressed_key_Q = input_helper.key_pressed(VirtualKeyCode::Q);
        let pressed_key_E = input_helper.key_pressed(VirtualKeyCode::E);
        let pressed_key_Z = input_helper.key_pressed(VirtualKeyCode::Z);
        let pressed_key_C = input_helper.key_pressed(VirtualKeyCode::C);

        let mut main_camera = scene_manager_data._main_camera.borrow_mut();
        let camera_move_speed = self._camera_move_speed * 5.0;

        // released_key_LeftBracket <- getKeyReleased keyboardInputData GLFW.Key'LeftBracket
        // released_key_RightBracket <- getKeyReleased keyboardInputData GLFW.Key'RightBracket
        // let mousePosDelta = _mousePosDelta mouseMoveData
        //     mousePosDeltaX = fromIntegral . unScalar $ (mousePosDelta .! Idx 0) :: Float
        //     mousePosDeltaY = fromIntegral . unScalar $ (mousePosDelta .! Idx 1) :: Float
        //     scroll_xoffset = _scroll_xoffset mouseMoveData
        //     scroll_yoffset = _scroll_yoffset mouseMoveData
        //     btn_left = _btn_l_down mouseInputData
        //     btn_middle = _btn_m_down mouseInputData
        //     btn_right = _btn_r_down mouseInputData
        let modifier_keys_shift = input_helper.key_held(VirtualKeyCode::LShift);
        let modified_camera_move_speed = camera_move_speed; // max 0.1 $ min 100.0 (cameraMoveSpeed + scroll_yoffset)
        let camera_move_speed_multiplier = if modifier_keys_shift { 2.0 } else { 1.0 } * modified_camera_move_speed;
        let move_speed: f32 = constants::CAMERA_MOVE_SPEED * camera_move_speed_multiplier * delta_time as f32;
        let pan_speed = constants::CAMERA_PAN_SPEED * camera_move_speed_multiplier;
        let rotation_speed = constants::CAMERA_ROTATION_SPEED;
        //
        // if released_key_LeftBracket then
        //     Renderer.prevDebugRenderTarget _rendererData
        // else when released_key_RightBracket $ do
        //     Renderer.nextDebugRenderTarget _rendererData
        //
        // when (0.0 /= scroll_yoffset) $
        //     writeIORef _cameraMoveSpeed modifiedCameraMoveSpeed

        if btn_left && btn_right {
            main_camera._transform_object.move_left(-pan_speed * mouse_delta_x);
            main_camera._transform_object.move_up(pan_speed * mouse_delta_y);
        }
        else if btn_right {
            main_camera._transform_object.rotation_pitch(-rotation_speed * mouse_delta_y);
            main_camera._transform_object.rotation_yaw(-rotation_speed * mouse_delta_x);
        }

        if pressed_key_Z {
            main_camera._transform_object.rotation_roll(-rotation_speed * delta_time as f32);
        }
        else if pressed_key_C {
            main_camera._transform_object.rotation_roll(rotation_speed * delta_time as f32);
        }

        if pressed_key_W {
            main_camera._transform_object.move_front(-move_speed);
        }
        else if pressed_key_S {
            main_camera._transform_object.move_front(move_speed);
        }

        if pressed_key_A {
            main_camera._transform_object.move_left(-move_speed);
        }
        else if pressed_key_D {
            main_camera._transform_object.move_left(move_speed);
        }

        if pressed_key_Q {
            main_camera._transform_object.move_up(-move_speed);
        }
        else if pressed_key_E {
            main_camera._transform_object.move_up(move_speed);
        }
    }
}


pub fn run_application(app_name: &str, app_version: u32, window_size: (u32, u32)) {
    log::info!("run_application");
    let mut input_helper = WinitInputHelper::new();
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
            if input_helper.update(&event) {
                if input_helper.key_released(VirtualKeyCode::Escape) || input_helper.quit() {
                    *control_flow = ControlFlow::Exit;
                    application_data.terminate_applicateion(
                        &mut scene_manager_data,
                        &mut resources.borrow_mut(),
                        &mut renderer_data,
                    );
                    run_application = false;
                    return;
                }
                application_data.update_event(&scene_manager_data, &input_helper);
            }

            match event {
                Event::MainEventsCleared => {
                    application_data._time_data.update_time_data(&time_instance);
                    let elapsed_time = application_data._time_data._elapsed_time;
                    let delta_time = application_data._time_data._delta_time;

                    if renderer_data.get_need_recreate_swapchain() {
                        if false == renderer_data.get_is_first_resize_event() {
                            renderer_data.resize_window();
                        }
                        let window_size = renderer_data._window.inner_size();
                        let aspect: f32 = if 0 != window_size.height {
                            window_size.width as f32 / window_size.height as f32
                        } else {
                            1.0
                        };
                        scene_manager_data.get_main_camera().borrow_mut().set_aspect(aspect);
                        renderer_data.set_is_first_resize_event(false);
                        renderer_data.set_need_recreate_swapchain(false);
                    }

                    scene_manager_data.update_scene_manager_data(elapsed_time, delta_time);
                    renderer_data.render_scene(scene_manager_data, elapsed_time, delta_time);
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                    },
                    WindowEvent::Resized { .. } => {
                        renderer_data.set_need_recreate_swapchain(true);
                    },
                    // WindowEvent::MouseInput { button: MouseButton::Left, state, .. } => {
                    //     if state == ElementState::Pressed {
                    //         is_left_clicked = Some(true);
                    //     } else {
                    //         is_left_clicked = Some(false);
                    //     }
                    // }
                    // WindowEvent::CursorMoved { position, .. } => {
                    //     let position: (i32, i32) = position.into();
                    //     cursor_position = Some([position.0, position.1]);
                    // }
                    // WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, v_lines), .. } => {
                    //     wheel_delta = Some(v_lines);
                    // }
                    WindowEvent::KeyboardInput { input, .. } => {
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::Escape) => {
                                /* keyboard event */
                            },
                            _ => { }
                        }
                    }
                    _ => { },
                },
                Event::RedrawEventsCleared => { },
                _ => { },
            }
        }
    });
}
