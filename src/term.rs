
fn run() -> Result<(), Box<dyn Error>> {
    let mut board = Board::<128, 64>::new();

    for _i in 0..1000 {
        print_board(&board);
        next_step(&mut board);
        std::thread::sleep(Duration::from_millis(250));
    }

    Ok(())
}

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