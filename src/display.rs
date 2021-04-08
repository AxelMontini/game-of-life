use glium::program;
use glium::texture::Texture2d;
use glium::Surface;
use glium::{texture::RawImage2d, Display, IndexBuffer, Program, VertexBuffer};
use rand::prelude::*;
use std::error::Error;
use std::time::Instant;

use crate::{INTERVAL_SEC, WIDTH};

#[cfg(feature = "cpucompute")]
use crate::cpu::{next_step, Board, Cell};

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

glium::implement_vertex!(Vertex, position, color);

#[cfg(feature = "cpucompute")]
pub fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_resizable(false)
        .with_inner_size(glutin::dpi::PhysicalSize::new(
            4 * WIDTH as u16,
            4 * WIDTH as u16,
        ))
        .with_title("Game of Life in Rust");
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop)?;

    let vertex_buffer = glium::VertexBuffer::new(
        &display,
        &[
            Vertex {
                color: [1.0, 1.0, 1.0],
                position: [0.0, 0.0],
            },
            Vertex {
                color: [1.0, 1.0, 1.0],
                position: [0.0, WIDTH as f32],
            },
            Vertex {
                color: [1.0, 1.0, 1.0],
                position: [WIDTH as f32, 0.0],
            },
            Vertex {
                color: [1.0, 1.0, 1.0],
                position: [WIDTH as f32, 0.0],
            },
            Vertex {
                color: [1.0, 1.0, 1.0],
                position: [0.0, WIDTH as f32],
            },
            Vertex {
                color: [1.0, 1.0, 1.0],
                position: [WIDTH as f32, WIDTH as f32],
            },
        ],
    )?;

    let index_buffer = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &[0u16, 1, 2, 3, 4, 5, 6],
    )?;

    let program = glium::program!(&display,
        140 => {
            vertex: include_str!("../display.vert"),

            fragment: include_str!("../display.frag")
        },
    )?;

    let mut board = Board::<WIDTH, WIDTH>::new();

    let mut last_step = Instant::now();

    // the main loop
    event_loop.run(move |event, _, control_flow| {
        let elapsed_dur = last_step.elapsed();

        if elapsed_dur.as_secs_f32() >= INTERVAL_SEC {
            println!("Rn");
            print_board(&board, &display, &vertex_buffer, &index_buffer, &program)
                .expect("printing board");
            next_step(&mut board);
            last_step = Instant::now();
        }

        *control_flow = match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,
                _ => glutin::event_loop::ControlFlow::Poll,
            },
            _ => glutin::event_loop::ControlFlow::Poll,
        };
    });
}

#[cfg(not(feature = "gpucompute"))]
fn print_board<const W: usize, const H: usize>(
    board: &Board<W, H>,
    display: &glium::Display,
    vertex_buffer: &glium::VertexBuffer<Vertex>,
    index_buffer: &glium::IndexBuffer<u16>,
    program: &glium::Program,
) -> Result<(), Box<dyn Error>> {
    use glium::uniform;

    let pixels: Vec<_> = board
        .board()
        .iter()
        .flatten()
        .flat_map(|cell| match cell {
            Cell::Alive => [255u8, 255, 255].iter().copied(),
            Cell::Dead => [0, 0, 0].iter().copied(),
        })
        .collect();

    let image_raw = RawImage2d::from_raw_rgb(pixels, (WIDTH as u32, WIDTH as u32));
    let texture = glium::texture::texture2d::Texture2d::new(display, image_raw)?;
    let texture_sampler = texture
        .sampled()
        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);

    let ortho_matrix: [[f32; 4]; 4] =
        cgmath::ortho::<f32>(0.0, WIDTH as f32, 0.0, WIDTH as f32, -1.0, 1.0).into();

    let uniforms = uniform! {
        matrix: ortho_matrix,
        grid: texture_sampler,
        scale: WIDTH as f32,
    };

    let mut target = display.draw();

    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(
        vertex_buffer,
        index_buffer,
        program,
        &uniforms,
        &Default::default(),
    )?;
    target.finish()?;

    Ok(())
}

#[cfg(feature = "gpucompute")]
pub fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_resizable(false)
        .with_inner_size(glutin::dpi::PhysicalSize::new(
            4 * WIDTH as u16,
            4 * WIDTH as u16,
        ))
        .with_title("Game of Life in Rust");
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop)?;

    let vertex_buffer = glium::VertexBuffer::new(
        &display,
        &[
            Vertex {
                color: [1.0, 1.0, 1.0],
                position: [0.0, 0.0],
            },
            Vertex {
                color: [1.0, 1.0, 1.0],
                position: [0.0, WIDTH as f32],
            },
            Vertex {
                color: [1.0, 1.0, 1.0],
                position: [WIDTH as f32, 0.0],
            },
            Vertex {
                color: [1.0, 1.0, 1.0],
                position: [WIDTH as f32, 0.0],
            },
            Vertex {
                color: [1.0, 1.0, 1.0],
                position: [0.0, WIDTH as f32],
            },
            Vertex {
                color: [1.0, 1.0, 1.0],
                position: [WIDTH as f32, WIDTH as f32],
            },
        ],
    )?;

    let index_buffer = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &[0u16, 1, 2, 3, 4, 5, 6],
    )?;

    let program_display = glium::program!(&display,
        140 => {
            vertex: include_str!("../display.vert"),

            fragment: include_str!("../display.frag")
        },
    )?;

    let program_compute = glium::program!(&display,
        140 => {
            vertex: include_str!("../display.vert"),

            fragment: include_str!("../compute.frag")
        },
    )?;

    // generate a random initial state
    let initial_data = {
        let mut rng = rand::thread_rng();
        (0..WIDTH * WIDTH)
            .flat_map(|_| {
                [[1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0]]
                    .choose(&mut rng)
                    .unwrap()
            })
            .copied()
            .collect()
    };

    // image used to display
    let old_image = RawImage2d::from_raw_rgba(initial_data, (WIDTH as u32, WIDTH as u32));
    let mut old_texture = Texture2d::new(&display, old_image)?;

    // texture used as intermediate result
    let new_image =
        RawImage2d::from_raw_rgba(vec![0.0; WIDTH * WIDTH * 4], (WIDTH as u32, WIDTH as u32));
    let mut new_texture = Texture2d::new(&display, new_image)?;

    let mut last_step = Instant::now();

    // the main loop
    event_loop.run(move |event, _, control_flow| {
        let elapsed_dur = last_step.elapsed();

        if elapsed_dur.as_secs_f32() >= INTERVAL_SEC {
            let data = crate::gpu::StepData {
                display: &display,
                vertex_buffer: &vertex_buffer,
                index_buffer: &index_buffer,
                program_display: &program_display,
                program_compute: &program_compute,
                new_texture: &new_texture,
                old_texture: &old_texture,
            };

            crate::gpu::step(data).expect("printing board");

            // swap textures after computation.
            std::mem::swap(&mut old_texture, &mut new_texture);

            last_step = Instant::now();
        }

        *control_flow = match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,
                _ => glutin::event_loop::ControlFlow::Poll,
            },
            _ => glutin::event_loop::ControlFlow::Poll,
        };
    });
}
