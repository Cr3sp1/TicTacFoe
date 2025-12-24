use std::fmt;

pub fn hello_world() {
    println!("Available marks:");
    println!("X: {}", Mark::X);
    println!("O: {}", Mark::O);
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mark {
    X,
    O,
}

impl fmt::Display for Mark {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mark::X => write!(f, "X"),
            Mark::O => write!(f, "O"),
        }
    }
}

pub struct Board {
    cells: [Option<Mark>; 9],
}

impl Board {
    pub fn new() -> Self {
        Board { cells: [None; 9] }
    }

    pub fn get(&self, row: usize, col: usize) -> Option<Mark> {
        if row > 3 || col > 3 {
            panic!("Tried to access board position ({row}, {col}) which is out of bounds");
        }
        self.cells[row * 3 + col]
    }

    pub fn set(&mut self, row: usize, col: usize, mark: Option<Mark>) {
        if row > 3 || col > 3 {
            panic!("Tried to access board position ({row}, {col}) which is out of bounds");
        }
        self.cells[row * 3 + col] = mark;
    }

    fn set_row(&mut self, row: usize, marks: [Option<Mark>; 3]) {
        for col in 0..3 {
            self.set(row, col, marks[col]);
        }
    }

    fn set_col(&mut self, col: usize, marks: [Option<Mark>; 3]) {
        for row in 0..3 {
            self.set(row, col, marks[row]);
        }
    }

    fn check_row(&self, row: usize) -> Option<Mark> {
        let mark_0 = self.get(row, 0)?;
        for i in 1..3 {
            let mark_i = self.get(row, i)?;
            if mark_i != mark_0 {
                return None;
            }
        }
        Some(mark_0)
    }

    fn check_col(&self, col: usize) -> Option<Mark> {
        let mark_0 = self.get(0, col)?;
        for i in 1..3 {
            let mark_i = self.get(i, col)?;
            if mark_i != mark_0 {
                return None;
            }
        }
        Some(mark_0)
    }

    fn check_diag_dexter(&self) -> Option<Mark> {
        let mark_0 = self.get(0, 0)?;
        for i in 1..3 {
            let mark_i = self.get(i, i)?;
            if mark_i != mark_0 {
                return None;
            }
        }
        Some(mark_0)
    }

    fn check_diag_sinister(&self) -> Option<Mark> {
        let mark_0 = self.get(0, 2)?;
        for i in 1..3 {
            let mark_i = self.get(i, 2 - i)?;
            if mark_i != mark_0 {
                return None;
            }
        }
        Some(mark_0)
    }

    pub fn check_all(&self) -> Option<Mark> {
        if let Some(mark) = self.check_diag_dexter() {
            return Some(mark);
        }
        if let Some(mark) = self.check_diag_sinister() {
            return Some(mark);
        }
        for i in 0..3 {
            if let Some(mark) = self.check_row(i) {
                return Some(mark);
            }
            if let Some(mark) = self.check_col(i) {
                return Some(mark);
            }
        }

        None
    }

    pub fn check_complete(&self) -> bool {
        for i in 0..9 {
            if self.cells[i].is_none() {
                return false;
            }
        }
        true
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..3 {
            for col in 0..3 {
                let index = row * 3 + col;
                match self.cells[index] {
                    Some(mark) => write!(f, " {} ", mark)?,
                    None => write!(f, " {} ", index)?,
                }
                if col < 2 {
                    write!(f, "|")?;
                }
            }
            if row < 2 {
                writeln!(f)?;
                writeln!(f, "-----------")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_row() {
        let mut board = Board::new();
        assert_eq!(board.check_row(0), None);

        board.set_row(0, [Some(Mark::X), Some(Mark::X), Some(Mark::X)]);
        assert_eq!(board.check_row(0), Some(Mark::X));

        board.set_row(1, [Some(Mark::O), Some(Mark::O), Some(Mark::O)]);
        assert_eq!(board.check_row(1), Some(Mark::O));

        board.set_row(0, [Some(Mark::X), Some(Mark::O), Some(Mark::X)]);
        assert_eq!(board.check_row(0), None);

        board.set(0, 1, Some(Mark::X));
        assert_eq!(board.check_row(0), Some(Mark::X));

        board.set_row(0, [Some(Mark::X), Some(Mark::O), None]);
        assert_eq!(board.check_row(0), None);
    }

    #[test]
    fn test_check_col() {
        let mut board = Board::new();
        assert_eq!(board.check_col(0), None);

        board.set_col(0, [Some(Mark::X), Some(Mark::X), Some(Mark::X)]);
        assert_eq!(board.check_col(0), Some(Mark::X));

        board.set_col(1, [Some(Mark::O), Some(Mark::O), Some(Mark::O)]);
        assert_eq!(board.check_col(1), Some(Mark::O));

        board.set_col(0, [Some(Mark::X), Some(Mark::O), Some(Mark::X)]);
        assert_eq!(board.check_col(0), None);

        board.set(1, 0, Some(Mark::X));
        assert_eq!(board.check_col(0), Some(Mark::X));

        board.set_col(0, [Some(Mark::X), Some(Mark::O), None]);
        assert_eq!(board.check_col(0), None);
    }

    #[test]
    fn test_check_diag() {
        let mut board = Board::new();
        assert_eq!(board.check_diag_dexter(), None);
        assert_eq!(board.check_diag_sinister(), None);

        board.set_row(0, [Some(Mark::X), Some(Mark::O), Some(Mark::O)]);
        board.set_row(1, [Some(Mark::X), None, Some(Mark::X)]);
        board.set_row(2, [Some(Mark::O), Some(Mark::X), Some(Mark::X)]);
        assert_eq!(board.check_diag_dexter(), None);
        assert_eq!(board.check_diag_sinister(), None);

        board.set(1, 1, Some(Mark::X));
        assert_eq!(board.check_diag_dexter(), Some(Mark::X));
        assert_eq!(board.check_diag_sinister(), None);

        board.set(1, 1, Some(Mark::O));
        assert_eq!(board.check_diag_dexter(), None);
        assert_eq!(board.check_diag_sinister(), Some(Mark::O));
    }

    #[test]
    fn test_check_all() {
        let mut board = Board::new();
        assert_eq!(board.check_diag_dexter(), None);
        assert_eq!(board.check_diag_sinister(), None);

        board.set_row(0, [Some(Mark::X), Some(Mark::O), Some(Mark::O)]);
        board.set_row(1, [Some(Mark::X), None, Some(Mark::X)]);
        board.set_row(2, [None, Some(Mark::O), None]);
        assert_eq!(board.check_all(), None);

        board.set(1, 1, Some(Mark::X));
        assert_eq!(board.check_all(), Some(Mark::X));

        board.set(1, 1, Some(Mark::O));
        assert_eq!(board.check_all(), Some(Mark::O));

        board.set(0, 1, Some(Mark::X));
        assert_eq!(board.check_all(), None);
        board.set(2, 0, Some(Mark::O));
        assert_eq!(board.check_all(), Some(Mark::O));
    }

    #[test]
    fn test_check_complete() {
        let mut board = Board::new();
        assert_eq!(board.check_complete(), false);

        board.set_row(0, [Some(Mark::X), Some(Mark::O), Some(Mark::O)]);
        board.set_row(1, [Some(Mark::X), None, Some(Mark::X)]);
        board.set_row(2, [Some(Mark::O), Some(Mark::O), Some(Mark::X)]);
        assert_eq!(board.check_complete(), false);

        board.set(1, 1, Some(Mark::X));
        assert_eq!(board.check_complete(), true);
    }
}
