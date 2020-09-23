// Copyright (c) 2017 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::device::Queue;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;

use std::sync::Arc;
use vulkano::pipeline::input_assembly::Index;

pub struct TriangleDrawSystem {
    gfx_queue: Arc<Queue>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u16]>>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}

impl TriangleDrawSystem {
    /// Initializes a triangle drawing system.
    pub fn new<R>(gfx_queue: Arc<Queue>, subpass: Subpass<R>) -> TriangleDrawSystem
    where
        R: RenderPassAbstract + Send + Sync + 'static,
    {
        let vertex_buffer = {
            CpuAccessibleBuffer::from_iter(
                gfx_queue.device().clone(),
                BufferUsage::all(),
                false,
                [
                    Vertex {
                        position: [-0.5, -0.5, 0.0],
                        normal: [1.0, 0.0, 0.0],
                        ..Vertex::default()
                    },
                    Vertex {
                        position: [-0.5, 0.5, 0.0],
                        normal: [1.0, 1.0, 0.0],
                        ..Vertex::default()
                    },
                    Vertex {
                        position: [0.5, -0.5, 0.0],
                        normal: [0.0, 0.0, 1.0],
                        ..Vertex::default()
                    },
                    Vertex {
                        position: [0.5, 0.5, 0.0],
                        normal: [1.0, 1.0, 1.0],
                        ..Vertex::default()
                    },
                ]
                .iter()
                .cloned(),
            )
            .expect("failed to create buffer")
        };

        pub const INDICES: [u16; 6] = [0, 1, 2, 1, 2, 3];
        let indices = INDICES.iter().cloned();
        let index_buffer = CpuAccessibleBuffer::from_iter(
                gfx_queue.device().clone(),
                BufferUsage::all(),
                false,
                indices
            ).unwrap();

        let pipeline = {
            let vs = vs::Shader::load(gfx_queue.device().clone())
                .expect("failed to create shader module");
            let fs = fs::Shader::load(gfx_queue.device().clone())
                .expect("failed to create shader module");

            Arc::new(
                GraphicsPipeline::start()
                    .vertex_input_single_buffer::<Vertex>()
                    .vertex_shader(vs.main_entry_point(), ())
                    .triangle_list()
                    .viewports_dynamic_scissors_irrelevant(1)
                    .fragment_shader(fs.main_entry_point(), ())
                    .depth_stencil_simple_depth()
                    .render_pass(subpass)
                    .build(gfx_queue.device().clone())
                    .unwrap(),
            ) as Arc<_>
        };

        TriangleDrawSystem {
            gfx_queue: gfx_queue,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            pipeline: pipeline,
        }
    }

    /// Builds a secondary command buffer that draws the triangle on the current subpass.
    pub fn draw(&self, viewport_dimensions: [u32; 2]) -> AutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
            self.pipeline.clone().subpass(),
        ).unwrap();
        builder.draw_indexed(
                self.pipeline.clone(),
                &DynamicState {
                    viewports: Some(vec![Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [viewport_dimensions[0] as f32, viewport_dimensions[1] as f32],
                        depth_range: 0.0..1.0,
                    }]),
                    ..DynamicState::none()
                },
                vec![self.vertex_buffer.clone()],
                self.index_buffer.clone(),
                (),
                (),
            ).unwrap();
        builder.build().unwrap()
    }
}

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tangent: [f32; 3],
    color: u32,
    tex_coord: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, normal, tangent, color, tex_coord);

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 tangent;
layout(location = 3) in uint color;
layout(location = 4) in vec2 tex_coord;

layout(location = 0) out vec3 v_color;

void main() {
    gl_Position = vec4(position, 1.0);
    v_color = normal;
}"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450

layout(location = 0) in vec3 v_color;

layout(location = 0) out vec4 f_color;
layout(location = 1) out vec3 f_normal;

void main() {
    f_color = vec4(v_color, 1.0);
    f_normal = vec3(0.0, 0.0, 1.0);
}"
    }
}
