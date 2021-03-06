use serde_json;

use crate::vulkan_context::render_pass::{
    RenderPassPipelineData,
    RenderPassPipelineDataMap,
};

#[derive(Clone, Debug)]
pub struct MaterialData {
    pub _material_data_name: String,
    pub _render_pass_pipeline_data_map: RenderPassPipelineDataMap,
    pub _material_parameter_map: serde_json::Value,
}

impl MaterialData {
    pub fn create_material(
        material_data_name: &String,
        render_pass_pipeline_datas: &Vec<RenderPassPipelineData>,
        material_parameter_map: &serde_json::Value
    ) -> MaterialData {
        log::debug!("create_material: {}", material_data_name);

        let mut render_pass_pipeline_data_map = RenderPassPipelineDataMap::new();
        for render_pass_pipeline_data in render_pass_pipeline_datas {
            let render_pass_pipeline_data_name = format!(
                "{}/{}",
                render_pass_pipeline_data._render_pass_data.borrow()._render_pass_data_name,
                render_pass_pipeline_data._pipeline_data.borrow()._pipeline_data_name
            );
            log::trace!("    renderPass/pipeline: {:?}", render_pass_pipeline_data_name);
            render_pass_pipeline_data_map.insert(render_pass_pipeline_data_name, render_pass_pipeline_data.clone());
        }
        MaterialData {
            _material_data_name: material_data_name.clone(),
            _render_pass_pipeline_data_map: render_pass_pipeline_data_map,
            _material_parameter_map: material_parameter_map.clone()
        }
    }

    pub fn destroy_material(&self) {
        log::debug!("create_material: {}", self._material_data_name);
    }

    pub fn get_render_pass_pipeline_data(
        &self,
        render_pass_pipeline_data_name: &str
    ) -> &RenderPassPipelineData {
        self._render_pass_pipeline_data_map.get(render_pass_pipeline_data_name).unwrap()
    }
}