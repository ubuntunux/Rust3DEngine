use std::cell::RefMut;
use std::time;
use log;

use nalgebra::{
    Vector3,
};
use winit::event::{
    ElementState,
    Event,
    MouseButton,
    MouseScrollDelta,
    Touch,
    TouchPhase,
    VirtualKeyCode,
    WindowEvent
};
use winit::event_loop::{
    ControlFlow,
    EventLoop
};
use winit::dpi;
use winit::window::{ WindowBuilder };

use crate::constants;
use crate::application::{ scene_manager, SceneManagerData };
use crate::application::input;
use crate::resource::{ self, Resources};
use crate::renderer::{ self, RendererData, CameraCreateInfo };
use crate::renderer::font::FontManager;
use crate::renderer::ui::UIManager;
use crate::utilities::system::{self, RcRefCell, newRcRefCell};
use crate::utilities::logger;

#[derive(Debug, Clone)]
pub struct TimeData {
    _acc_frame_time: f64,
    _acc_frame_count: i32,
    _elapsed_frame: u64,
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
        _elapsed_frame: 0,
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
        self._elapsed_frame += 1;
        if 1.0 < acc_frame_time {
            let average_frame_time = acc_frame_time / (acc_frame_count as f64) * 1000.0;
            let average_fps = 1000.0 / average_frame_time;
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
    _window_size: (u32, u32),
    _time_data: TimeData,
    _camera_move_speed: f32,
    _keyboard_input_data: Box<input::KeyboardInputData>,
    _mouse_move_data: Box<input::MouseMoveData>,
    _mouse_input_data: Box<input::MouseInputData>,
    _scene_manager_data: RcRefCell<scene_manager::SceneManagerData>,
    _renderer_data: RcRefCell<renderer::RendererData>,
    _font_manager: RcRefCell<FontManager>,
    _ui_manager: RcRefCell<UIManager>,
    _resources: RcRefCell<resource::Resources>
}

impl ApplicationData {
    pub fn terminate_applicateion(
        &mut self,
        font_manager: &mut FontManager,
        ui_manager: &mut UIManager,
        scene_manager_data: &mut SceneManagerData,
        resources: &mut Resources,
        renderer_data: &mut RendererData,
    ) {
        scene_manager_data.close_scene_manager_data(renderer_data.get_device());
        ui_manager.destroy_ui_manager(renderer_data.get_device());
        font_manager.destroy_font_manager(renderer_data.get_device());
        renderer_data.destroy_framebuffer_and_descriptors();
        resources.destroy_resources(renderer_data);
        renderer_data.destroy_renderer_data();
    }

    pub fn clear_input_events(&mut self) {
        self._mouse_move_data.clear_mouse_move_delta();
        self._mouse_input_data.clear_mouse_input();
        self._keyboard_input_data.clear_key_pressed();
        self._keyboard_input_data.clear_key_released();
    }

    pub fn update_event(&mut self, scene_manager_data: &SceneManagerData) {
        let renderer_data: *mut RendererData = scene_manager_data._renderer_data.as_ptr();

        const MOUSE_DELTA_RATIO: f32 = 500.0;
        let delta_time = self._time_data._delta_time;
        let _mouse_pos = &self._mouse_move_data._mouse_pos;
        let mouse_delta_x = self._mouse_move_data._mouse_pos_delta.x as f32 / self._window_size.0 as f32 * MOUSE_DELTA_RATIO;
        let mouse_delta_y = self._mouse_move_data._mouse_pos_delta.y as f32 / self._window_size.1 as f32 * MOUSE_DELTA_RATIO;
        let btn_left: bool = self._mouse_input_data._btn_l_hold;
        let btn_right: bool = self._mouse_input_data._btn_r_hold;
        let _btn_middle: bool = self._mouse_input_data._btn_m_hold;

        let pressed_key_a = self._keyboard_input_data.get_key_hold(VirtualKeyCode::A);
        let pressed_key_d = self._keyboard_input_data.get_key_hold(VirtualKeyCode::D);
        let pressed_key_w = self._keyboard_input_data.get_key_hold(VirtualKeyCode::W);
        let pressed_key_s = self._keyboard_input_data.get_key_hold(VirtualKeyCode::S);
        let pressed_key_q = self._keyboard_input_data.get_key_hold(VirtualKeyCode::Q);
        let pressed_key_e = self._keyboard_input_data.get_key_hold(VirtualKeyCode::E);
        let pressed_key_z = self._keyboard_input_data.get_key_hold(VirtualKeyCode::Z);
        let pressed_key_c = self._keyboard_input_data.get_key_hold(VirtualKeyCode::C);
        let pressed_key_comma = self._keyboard_input_data.get_key_hold(VirtualKeyCode::Comma);
        let pressed_key_period = self._keyboard_input_data.get_key_hold(VirtualKeyCode::Period);
        let released_key_left_bracket = self._keyboard_input_data.get_key_released(VirtualKeyCode::LBracket);
        let released_key_right_bracket = self._keyboard_input_data.get_key_released(VirtualKeyCode::RBracket);
        let released_key_subtract = self._keyboard_input_data.get_key_released(VirtualKeyCode::Minus);
        let released_key_equals = self._keyboard_input_data.get_key_released(VirtualKeyCode::Equals);

        let mut main_camera = scene_manager_data._main_camera.borrow_mut();
        let mut main_light = scene_manager_data._main_light.borrow_mut();
        let camera_move_speed = self._camera_move_speed;
        let modifier_keys_shift = self._keyboard_input_data.get_key_hold(VirtualKeyCode::LShift);
        let modified_camera_move_speed = camera_move_speed; // max 0.1 $ min 100.0 (cameraMoveSpeed + scroll_yoffset)
        let camera_move_speed_multiplier = if modifier_keys_shift { 2.0 } else { 1.0 } * modified_camera_move_speed;
        let move_speed: f32 = constants::CAMERA_MOVE_SPEED * camera_move_speed_multiplier * delta_time as f32;
        let pan_speed = constants::CAMERA_PAN_SPEED * camera_move_speed_multiplier;
        let _rotation_speed = constants::CAMERA_ROTATION_SPEED;

        unsafe {
            if released_key_left_bracket {
                (*renderer_data).prev_debug_render_target();
            } else if released_key_right_bracket {
                (*renderer_data).next_debug_render_target();
            }

            if released_key_subtract {
                (*renderer_data).prev_debug_render_target_miplevel();
            } else if released_key_equals {
                (*renderer_data).next_debug_render_target_miplevel();
            }
        }

        #[cfg(target_os = "android")]
        let rotation_speed = 0.02 * delta_time as f32;
        #[cfg(not(target_os = "android"))]
        let rotation_speed = delta_time as f32;

        if pressed_key_comma {
            main_light._transform_object.rotation_pitch(rotation_speed);
        } else if pressed_key_period {
            main_light._transform_object.rotation_pitch(-rotation_speed);
        }

        // when (0.0 /= scroll_yoffset) $
        //     writeIORef _cameraMoveSpeed modifiedCameraMoveSpeed

        if btn_left && btn_right {
            main_camera._transform_object.move_left(-pan_speed * mouse_delta_x as f32);
            main_camera._transform_object.move_up(pan_speed * mouse_delta_y as f32);
        }
        else if btn_right {
            main_camera._transform_object.rotation_pitch(-rotation_speed * mouse_delta_y as f32);
            main_camera._transform_object.rotation_yaw(-rotation_speed * mouse_delta_x as f32);
        }

        if pressed_key_z {
            main_camera._transform_object.rotation_roll(-rotation_speed * delta_time as f32);
        }
        else if pressed_key_c {
            main_camera._transform_object.rotation_roll(rotation_speed * delta_time as f32);
        }

        if pressed_key_w {
            main_camera._transform_object.move_front(-move_speed);
        }
        else if pressed_key_s {
            main_camera._transform_object.move_front(move_speed);
        }

        if pressed_key_a {
            main_camera._transform_object.move_left(-move_speed);
        }
        else if pressed_key_d {
            main_camera._transform_object.move_left(move_speed);
        }

        if pressed_key_q {
            main_camera._transform_object.move_up(-move_speed);
        }
        else if pressed_key_e {
            main_camera._transform_object.move_up(move_speed);
        }
    }
}

pub fn run_application() {
    logger::initialize_logger();

    log::info!("run_application");

    let app_name: &str = "RustEngine3D";
    let app_version: u32 = 1;
    let initial_window_size: (u32, u32) = (1024, 768);

    let time_instance = time::Instant::now();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(app_name)
        .with_inner_size(dpi::Size::Physical(dpi::PhysicalSize { width: initial_window_size.0, height: initial_window_size.1 }))
        .build(&event_loop)
        .unwrap();
    let window_size: (u32, u32) = (window.inner_size().width, window.inner_size().height);

    let mut maybe_resources: Option<RcRefCell<Resources>> = None;
    let mut maybe_ui_manager: Option<RcRefCell<UIManager>> = None;
    let mut maybe_font_manager: Option<RcRefCell<FontManager>> = None;
    let mut maybe_renderer_data: Option<RcRefCell<RendererData>> = None;
    let mut maybe_scene_manager_data: Option<RcRefCell<SceneManagerData>> = None;
    let mut maybe_application_data: Option<RcRefCell<ApplicationData>> = None;

    // main loop
    #[cfg(target_os = "android")]
    let mut need_initialize: bool = false;
    #[cfg(not(target_os = "android"))]
    let mut need_initialize: bool = true;
    let mut initialize_done: bool = false;
    let mut run_application: bool = false;
    event_loop.run(move |event, __window_target, control_flow|{
        if need_initialize {
            let elapsed_time = time_instance.elapsed().as_secs_f64();
            let (width, height) = window_size;
            let mouse_pos = (width / 2, height / 2);
            let resources = resource::create_resources();
            let font_manager: RcRefCell<FontManager> = newRcRefCell(FontManager::create_font_manager());
            let ui_manager: RcRefCell<UIManager> = newRcRefCell(UIManager::create_ui_manager());
            let renderer_data: RcRefCell<RendererData> = renderer::create_renderer_data(app_name, app_version, window_size, &window, resources.clone());
            let scene_manager_data = scene_manager::create_scene_manager_data(renderer_data.clone(), resources.clone());
            let keyboard_input_data = input::create_keyboard_input_data();
            let mouse_move_data = input::create_mouse_move_data(mouse_pos);
            let mouse_input_data = input::create_mouse_input_data();

            // initialize grphics
            scene_manager_data.borrow().get_fft_ocean().borrow_mut().regist_fft_ocean_textures(&renderer_data, &resources);
            resources.borrow_mut().initialize_resources(&mut renderer_data.borrow_mut());
            font_manager.borrow_mut().initialize_font_manager(&renderer_data.borrow(), &resources.borrow());
            ui_manager.borrow_mut().initialize_ui_manager(&renderer_data.borrow(), &resources.borrow());
            renderer_data.borrow_mut().prepare_framebuffer_and_descriptors();
            let camera_data = CameraCreateInfo {
                window_width: width,
                window_height: height,
                position: Vector3::new(-25.28, 18.20, 24.5), // Vector3::new(-7.29, 6.345, -0.33),
                rotation: Vector3::new(-0.157, -1.0, 0.0), // Vector3::new(-0.287, -1.5625, 0.0),
                ..Default::default()
            };
            scene_manager_data.borrow_mut().initialize_scene_graphics_data(&renderer_data.borrow());
            scene_manager_data.borrow_mut().open_scene_manager_data(&camera_data);

            let application_data = system::newRcRefCell(
                ApplicationData {
                    _window_size: window_size,
                    _time_data: create_time_data(elapsed_time),
                    _camera_move_speed: 1.0,
                    _keyboard_input_data: keyboard_input_data.clone(),
                    _mouse_move_data: mouse_move_data.clone(),
                    _mouse_input_data: mouse_input_data.clone(),
                    _font_manager: font_manager.clone(),
                    _ui_manager: ui_manager.clone(),
                    _scene_manager_data: scene_manager_data.clone(),
                    _renderer_data: renderer_data.clone(),
                    _resources: resources.clone(),
                }
            );

            maybe_resources = Some(resources);
            maybe_font_manager = Some(font_manager);
            maybe_ui_manager = Some(ui_manager);
            maybe_renderer_data = Some(renderer_data);
            maybe_scene_manager_data = Some(scene_manager_data);
            maybe_application_data = Some(application_data);
            run_application = true;
            need_initialize = false;
            initialize_done = true;
        }

        match event {
            Event::Resumed => {
                log::info!("Application was resumed");
                #[cfg(target_os = "android")]
                if false == initialize_done {
                    need_initialize = true;
                }
            },
            Event::Suspended => {
                log::info!("Application was suspended");
                #[cfg(target_os = "android")]
                if run_application {
                    {
                        // Destroy app on suspend for android target.
                        let mut application_data: RefMut<ApplicationData> = maybe_application_data.as_ref().unwrap().borrow_mut();
                        let mut renderer_data: RefMut<RendererData> = maybe_renderer_data.as_ref().unwrap().borrow_mut();
                        let mut scene_manager_data: RefMut<SceneManagerData> = maybe_scene_manager_data.as_ref().unwrap().borrow_mut();
                        let mut font_manager: RefMut<FontManager> = maybe_font_manager.as_ref().unwrap().borrow_mut();
                        let mut ui_manager: RefMut<UIManager> = maybe_ui_manager.as_ref().unwrap().borrow_mut();
                        application_data.terminate_applicateion(
                            &mut font_manager,
                            &mut ui_manager,
                            &mut scene_manager_data,
                            &mut maybe_resources.as_ref().unwrap().borrow_mut(),
                            &mut renderer_data,
                        );
                    }

                    maybe_resources = None;
                    maybe_ui_manager = None;
                    maybe_font_manager = None;
                    maybe_renderer_data = None;
                    maybe_scene_manager_data = None;
                    maybe_application_data = None;

                    run_application = false;
                    initialize_done = false;
                }
            },
            Event::NewEvents(_) => {
                // reset input states on new frame
                if run_application {
                    let mut application_data: RefMut<ApplicationData> = maybe_application_data.as_ref().unwrap().borrow_mut();
                    application_data.clear_input_events();
                }
            },
            Event::MainEventsCleared => {
                if run_application {
                    let mut application_data: RefMut<ApplicationData> = maybe_application_data.as_ref().unwrap().borrow_mut();
                    let mut renderer_data: RefMut<RendererData> = maybe_renderer_data.as_ref().unwrap().borrow_mut();
                    let mut scene_manager_data: RefMut<SceneManagerData> = maybe_scene_manager_data.as_ref().unwrap().borrow_mut();
                    let mut font_manager: RefMut<FontManager> = maybe_font_manager.as_ref().unwrap().borrow_mut();
                    let mut ui_manager: RefMut<UIManager> = maybe_ui_manager.as_ref().unwrap().borrow_mut();

                    // exit
                    if application_data._keyboard_input_data.get_key_pressed(VirtualKeyCode::Escape) {
                        *control_flow = ControlFlow::Exit;
                        application_data.terminate_applicateion(
                            &mut font_manager,
                            &mut ui_manager,
                            &mut scene_manager_data,
                            &mut maybe_resources.as_ref().unwrap().borrow_mut(),
                            &mut renderer_data,
                        );
                        run_application = false;
                        return;
                    }

                    // update event
                    application_data.update_event(&scene_manager_data);

                    // update timer
                    application_data._time_data.update_time_data(&time_instance);
                    let elapsed_time = application_data._time_data._elapsed_time;
                    let delta_time = application_data._time_data._delta_time;
                    let elapsed_frame = application_data._time_data._elapsed_frame;

                    font_manager.log(format!("{:.2}fps / {:.3}ms", application_data._time_data._average_fps, application_data._time_data._average_frame_time));

                    // update && render
                    if renderer_data.get_need_recreate_swapchain() {
                        log::info!("<<begin recreate_swapchain>>");

                        // destroy
                        scene_manager_data.destroy_scene_graphics_data(renderer_data.get_device());
                        ui_manager.destroy_ui_descriptor_sets();
                        font_manager.destroy_font_descriptor_sets();

                        renderer_data.resize_window();

                        // recreate
                        font_manager.create_font_descriptor_sets(&renderer_data, &renderer_data._resources.borrow());
                        ui_manager.create_ui_descriptor_sets(&renderer_data, &renderer_data._resources.borrow());
                        scene_manager_data.initialize_scene_graphics_data(&renderer_data);
                        renderer_data.set_need_recreate_swapchain(false);

                        log::info!("<<end recreate_swapchain>>");
                    }

                    renderer_data.update_post_process_datas();
                    scene_manager_data.update_scene_manager_data(elapsed_time, delta_time);
                    font_manager.update();
                    ui_manager.update(delta_time);
                    renderer_data.render_scene(scene_manager_data, &mut font_manager, &mut ui_manager, elapsed_time, delta_time, elapsed_frame);
                }
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                },
                WindowEvent::Resized(size) => {
                    log::info!("WindowEvent::Resized: {:?}, initialize_done: {}", size, initialize_done);
                    if initialize_done {
                        let mut application_data: RefMut<ApplicationData> = maybe_application_data.as_ref().unwrap().borrow_mut();
                        let scene_manager_data: RefMut<SceneManagerData> = maybe_scene_manager_data.as_ref().unwrap().borrow_mut();
                        let mut renderer_data: RefMut<RendererData> = maybe_renderer_data.as_ref().unwrap().borrow_mut();
                        application_data._window_size = (size.width, size.height);
                        scene_manager_data.get_main_camera().borrow_mut().set_aspect(size.width, size.height);
                        let swapchain_extent = renderer_data._swapchain_data._swapchain_extent;
                        let need_recreate_swapchain = swapchain_extent.width != size.width || swapchain_extent.height != size.height;
                        log::info!("need_recreate_swapchain: {}, swapchain_extent: {:?}", need_recreate_swapchain, swapchain_extent);
                        if need_recreate_swapchain {
                            renderer_data.set_need_recreate_swapchain(true);
                        }
                    }
                },
                WindowEvent::MouseInput { button, state, .. } => {
                    let mut application_data: RefMut<ApplicationData> = maybe_application_data.as_ref().unwrap().borrow_mut();
                    let mouse_input_data = &mut application_data._mouse_input_data;
                    let pressed = state == ElementState::Pressed;
                    match button {
                        MouseButton::Left => mouse_input_data.btn_l_pressed(pressed),
                        MouseButton::Middle => mouse_input_data.btn_m_pressed(pressed),
                        MouseButton::Right => mouse_input_data.btn_r_pressed(pressed),
                        _ => (),
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let mut application_data: RefMut<ApplicationData> = maybe_application_data.as_ref().unwrap().borrow_mut();
                    application_data._mouse_move_data.update_mouse_move(&position.into());
                }
                WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, _v_lines), .. } => {
                    // wheel_delta = Some(v_lines);
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if run_application {
                        let mut application_data: RefMut<ApplicationData> = maybe_application_data.as_ref().unwrap().borrow_mut();
                        match input.virtual_keycode {
                            Some(key) => {
                                if ElementState::Pressed == input.state {
                                    application_data._keyboard_input_data.set_key_pressed(key);
                                } else {
                                    application_data._keyboard_input_data.set_key_released(key);
                                }
                            }
                            None => {}
                        }
                    }
                }
                WindowEvent::Touch(Touch { device_id: _device_id, phase, location, force: _force, id }) => {
                    let mut application_data: RefMut<ApplicationData> = maybe_application_data.as_ref().unwrap().borrow_mut();

                    if 0 == id {
                        application_data._mouse_move_data.update_mouse_move(&location.into());

                        if phase == TouchPhase::Started {
                            application_data._mouse_input_data.btn_r_pressed(true);
                            application_data._mouse_move_data.clear_mouse_move_delta();
                        } else if phase == TouchPhase::Ended {
                            application_data._mouse_input_data.btn_r_pressed(false);
                        }
                    } else if 1 == id {
                        if phase == TouchPhase::Started {
                            application_data._keyboard_input_data.set_key_pressed(VirtualKeyCode::W);
                        } else if phase == TouchPhase::Ended {
                            application_data._keyboard_input_data.set_key_released(VirtualKeyCode::W);
                        }
                    }
                }
                _ => (),
            },
            Event::RedrawEventsCleared => {
            },
            Event::LoopDestroyed => {
                log::trace!("Application destroyed");
            }
            _ => (),
        }
    });
}
