use rayon::prelude::*;
use std::convert::TryInto;

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
