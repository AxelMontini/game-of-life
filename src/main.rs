use std::{
    convert::TryInto,
    error::Error,
    time::{Duration, Instant},
};

use cgmath::Matrix;
use glium::{texture::RawImage2d, Surface};
use rayon::prelude::*;

const WIDTH: usize = 256;
const INTERVAL_SEC: f32 = 0.1;

fn main() -> Result<(), Box<dyn Error>> {
    run()
}

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

glium::implement_vertex!(Vertex, position, color);

#[cfg(feature = "window")]
fn run() -> Result<(), Box<dyn Error>> {
    use glium::program;

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
            vertex: "
                #version 140
                uniform mat4 matrix;
                in vec2 position;
                out vec2 vCoords;
                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    vCoords = position;
                }
            ",

            fragment: &format!("
                #version 140
                uniform sampler2D grid;
                in vec2 vCoords;
                out vec4 f_color;
                void main() {{
                    f_color = texture(grid, vCoords / {});
                }}
            ", WIDTH as f32)
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

#[cfg(feature = "window")]
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

#[cfg(feature = "term")]
fn run() -> Result<(), Box<dyn Error>> {
    let mut board = Board::<128, 64>::new();

    for _i in 0..1000 {
        print_board(&board);
        next_step(&mut board);
        std::thread::sleep(Duration::from_millis(250));
    }

    Ok(())
}

#[cfg(feature = "term")]
fn print_board<const W: usize, const H: usize>(board: &Board<W, H>) {
    use crossterm::{
        cursor::{DisableBlinking, Hide, MoveTo},
        execute,
        style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
        terminal::{Clear, ClearType, SetSize},
    };
    use std::io::stdout;

    // reset cursor to starting position before doing anything else
    execute!(
        stdout(),
        SetSize(W as u16, H as u16),
        Clear(ClearType::All),
        Hide,
    )
    .expect("resetting cursor position");

    board.board().iter().for_each(|row| {
        execute!(
            stdout(),
            SetBackgroundColor(Color::White),
            SetForegroundColor(Color::Blue)
        )
        .expect("writing color to term");
        row.iter().for_each(|cell| {
            execute!(
                stdout(),
                Print(match cell {
                    Cell::Alive => "■",
                    Cell::Dead => " ",
                })
            )
            .expect("writing cell to term")
        });
        execute!(stdout(), ResetColor, Print("\n")).expect("writing newline to term");
    });
}

pub fn next_step<const W: usize, const H: usize>(board: &mut Board<W, H>) {
    // copy entire board first
    let view = *board.board();

    // iterate on view and modify board
    *board.board_mut() = (0..H)
        .into_par_iter()
        .map(|h| {
            let mut row = view[h];
            (0..W).for_each(|w| {
                let neighbors = neighbors(&view, h, w);

                row[w] = match (view[h][w], neighbors) {
                    (Cell::Dead, 3) => Cell::Alive,
                    (Cell::Alive, 2..=3) => Cell::Alive,
                    _ => Cell::Dead,
                };
            });

            row
        })
        .collect::<Vec<_>>()
        .as_slice()
        .try_into()
        .unwrap();
}

pub fn neighbors<const W: usize, const H: usize>(
    view: &[[Cell; W]; H],
    h: usize,
    w: usize,
) -> usize {
    (w.saturating_sub(1)..=(w + 1))
        .map(|c| {
            (h.saturating_sub(1)..=(h + 1))
                .map(move |r| {
                    view.get(r)
                        .and_then(|row| row.get(c))
                        .map_or(0, move |&cell| cell as u8)
                })
                .sum::<u8>()
        })
        .sum::<u8>()
        .saturating_sub(view[h][w] as u8) as usize
}

pub struct Board<const W: usize, const H: usize> {
    board: [[Cell; W]; H],
}

impl<const W: usize, const H: usize> Board<W, H> {
    pub fn new() -> Self {
        use rand::Rng;
        let mut board = [[Cell::Dead; W]; H];
        let mut rng = rand::thread_rng();

        let alive = rng.gen_range(0..(W * H));

        (0..alive).for_each(|_| {
            let w = rng.gen_range(0..W);
            let h = rng.gen_range(0..H);

            board[h][w] = Cell::Alive;
        });

        Self::with_board(board)
    }

    pub fn with_board(board: [[Cell; W]; H]) -> Self {
        Self { board }
    }

    pub fn board(&self) -> &[[Cell; W]; H] {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut [[Cell; W]; H] {
        &mut self.board
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cell {
    Alive = 1,
    Dead = 0,
}

#[cfg(test)]
mod tests {
    use super::{neighbors, next_step, Board, Cell};

    #[test]
    fn step_alive() {
        let board = [[Cell::Alive; 3]; 3];
        let mut board = Board::with_board(board);

        next_step(&mut board);

        let expected = [
            [Cell::Alive, Cell::Dead, Cell::Alive],
            [Cell::Dead, Cell::Dead, Cell::Dead],
            [Cell::Alive, Cell::Dead, Cell::Alive],
        ];

        assert_eq!(&expected, board.board());
    }

    #[test]
    fn reproduction() {
        // Center cell should become alive
        let board = [
            [Cell::Alive, Cell::Dead, Cell::Alive],
            [Cell::Dead, Cell::Dead, Cell::Dead],
            [Cell::Dead, Cell::Dead, Cell::Alive],
        ];
        let mut board = Board::with_board(board);

        next_step(&mut board);

        let expected = [
            [Cell::Dead, Cell::Dead, Cell::Dead],
            [Cell::Dead, Cell::Alive, Cell::Dead],
            [Cell::Dead, Cell::Dead, Cell::Dead],
        ];

        assert_eq!(&expected, board.board());
    }

    #[test]
    fn neighbors_count() {
        let view = [
            [Cell::Alive, Cell::Dead, Cell::Dead],
            [Cell::Alive, Cell::Dead, Cell::Alive],
            [Cell::Alive, Cell::Alive, Cell::Alive],
        ];

        assert_eq!(1, neighbors(&view, 0, 0));
        assert_eq!(3, neighbors(&view, 0, 1));
        assert_eq!(1, neighbors(&view, 0, 2));
        assert_eq!(3, neighbors(&view, 1, 0));
        assert_eq!(6, neighbors(&view, 1, 1));
        assert_eq!(2, neighbors(&view, 1, 2));
        assert_eq!(2, neighbors(&view, 2, 0));
        assert_eq!(4, neighbors(&view, 2, 1));
        assert_eq!(2, neighbors(&view, 2, 2));
    }
}
