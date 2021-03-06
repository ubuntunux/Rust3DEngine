use ash::Device;
use nalgebra::Vector2;

use crate::application::application::TimeData;
use crate::renderer::font::FontManager;
use crate::renderer::renderer::RendererData;
use crate::resource::resource::Resources;
use crate::utilities::system::RcRefCell;

pub trait ProjectSceneManagerBase {
    fn initialize_project_scene_manager(
        &mut self,
        scene_manager_data: &SceneManagerData,
        renderer_data: &RendererData,
        resources: &Resources,
        window_size: &Vector2<i32>,
    );
    fn initialize_scene_graphics_data(&self);
    fn destroy_scene_graphics_data(&self, device: &Device);
    fn get_window_size(&self) -> &Vector2<i32>;
    fn set_window_size(&mut self, width: i32, height: i32);
    fn resized_window(&mut self, width: i32, height: i32);
    fn create_default_scene_data(&self, scene_data_name: &str);
    fn open_scene_data(&mut self, scene_data_name: &str);
    fn close_scene_data(&mut self, device: &Device);
    fn save_scene_data(&mut self);
    fn destroy_project_scene_manager(&mut self, device: &Device);
    fn update_project_scene_manager(&mut self, time_data: &TimeData, font_manager: &mut FontManager);
}

pub struct SceneManagerData {
    pub _renderer_data: RcRefCell<RendererData>,
    pub _resources: RcRefCell<Resources>,
    pub _project_scene_manager: *const dyn ProjectSceneManagerBase,
}

impl SceneManagerData {
    pub fn create_scene_manager_data(
        renderer_data: &RcRefCell<RendererData>,
        resources: &RcRefCell<Resources>,
        project_scene_manager: *const dyn ProjectSceneManagerBase
    ) -> SceneManagerData {
        SceneManagerData {
            _renderer_data: renderer_data.clone(),
            _resources: resources.clone(),
            _project_scene_manager: project_scene_manager,
        }
    }

    pub fn initialize_scene_manager_data(
        &mut self,
        window_size: &Vector2<i32>,
        renderer_data: &RendererData,
        resources: &Resources
    ) {
        self.get_project_scene_manager_mut().initialize_project_scene_manager(
            self,
            renderer_data,
            resources,
            window_size
        );
    }

    pub fn get_project_scene_manager(&self) -> &dyn ProjectSceneManagerBase {
        unsafe { &*self._project_scene_manager }
    }

    pub fn get_project_scene_manager_mut(&self) -> &mut dyn ProjectSceneManagerBase {
        unsafe { &mut *(self._project_scene_manager as *mut dyn ProjectSceneManagerBase) }
    }

    pub fn open_scene_data(&mut self) {
        self.get_project_scene_manager_mut().open_scene_data("default");
    }

    pub fn close_scene_data(&mut self, device: &Device) {
        self.get_project_scene_manager_mut().close_scene_data(device);
    }

    pub fn save_scene_data(&mut self) {
        self.get_project_scene_manager_mut().save_scene_data();
    }

    pub fn destroy_scene_manager_data(&mut self, device: &Device) {
        self.get_project_scene_manager_mut().destroy_project_scene_manager(device);
    }

    pub fn initialize_scene_graphics_data(&self) {
        self.get_project_scene_manager_mut().initialize_scene_graphics_data();
    }

    pub fn destroy_scene_graphics_data(&self, device: &Device) {
        self.get_project_scene_manager_mut().destroy_scene_graphics_data(device);
    }

    pub fn resized_window(&self, width: i32, height: i32) {
        self.get_project_scene_manager_mut().resized_window(width, height);
    }

    pub fn update_scene_manager_data(&self, time_data: &TimeData, font_manager: &mut FontManager) {
        self.get_project_scene_manager_mut().update_project_scene_manager(time_data, font_manager);
    }
}