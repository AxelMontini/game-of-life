use std::error::Error;

use glium::{texture::Texture2d, uniform, Display, IndexBuffer, Program, Surface, VertexBuffer};

use crate::display::Vertex;
use crate::WIDTH;

pub struct StepData<'s> {
    pub display: &'s Display,
    pub vertex_buffer: &'s VertexBuffer<Vertex>,
    pub index_buffer: &'s IndexBuffer<u16>,
    pub program_display: &'s Program,
    pub program_compute: &'s Program,
    pub new_texture: &'s Texture2d,
    pub old_texture: &'s Texture2d,
}

pub fn step(data: StepData<'_>) -> Result<(), Box<dyn Error>> {
    let StepData {
        display,
        vertex_buffer,
        index_buffer,
        program_display,
        program_compute,
        new_texture,
        old_texture,
    } = data;

    let ortho_matrix: [[f32; 4]; 4] =
        cgmath::ortho::<f32>(0.0, WIDTH as f32, 0.0, WIDTH as f32, -1.0, 1.0).into();

    let compute_uniforms = uniform! {
        matrix: ortho_matrix,
        grid: new_texture,
        scale: WIDTH as f32,
    };

    let display_uniforms = uniform! {
        matrix: ortho_matrix,
        grid: new_texture,
        scale: WIDTH as f32,
    };

    // render onto new_texture first
    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(display, new_texture)?;

    framebuffer.draw(
        vertex_buffer,
        index_buffer,
        program_compute,
        &compute_uniforms,
        &Default::default(),
    )?;

    let mut target = display.draw();

    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(
        vertex_buffer,
        index_buffer,
        program_display,
        &display_uniforms,
        &Default::default(),
    )?;
    target.finish()?;

    Ok(())
}
