use crate::game::Board;

pub struct Position {
    pub row: usize,
    pub col: usize,
}

/// Moves selection left, wrapping to the rightmost column and finding
/// the next playable position.
pub fn move_selection_left_playable(board: &impl Board, selected: &mut Position) {
    for _ in 0..3 {
        let original_row = selected.row;
        move_selection_left(selected);

        for _ in 0..3 {
            if board.is_playable(selected.row, selected.col) {
                return;
            }
            match original_row {
                0 => move_selection_down(selected),
                2 => move_selection_up(selected),
                _ => move_selection_down(selected),
            }
        }
    }
}

/// Moves selection right, wrapping to the leftmost column and finding
/// the next playable position.
pub fn move_selection_right_playable(board: &impl Board, selected: &mut Position) {
    for _ in 0..3 {
        let original_row = selected.row;
        move_selection_right(selected);

        for _ in 0..3 {
            if board.is_playable(selected.row, selected.col) {
                return;
            }
            match original_row {
                0 => move_selection_down(selected),
                2 => move_selection_up(selected),
                _ => move_selection_down(selected),
            }
        }
    }
}

/// Moves selection up, wrapping to the bottom row and finding
/// the next playable position.
pub fn move_selection_up_playable(board: &impl Board, selected: &mut Position) {
    for _ in 0..3 {
        let original_col = selected.col;
        move_selection_up(selected);

        for _ in 0..3 {
            if board.is_playable(selected.row, selected.col) {
                return;
            }
            match original_col {
                0 => move_selection_right(selected),
                2 => move_selection_left(selected),
                _ => move_selection_right(selected),
            }
        }
    }
}

/// Moves selection down, wrapping to the top row and finding
/// the next playable position.
pub fn move_selection_down_playable(board: &impl Board, selected: &mut Position) {
    for _ in 0..3 {
        let original_col = selected.col;
        move_selection_down(selected);

        for _ in 0..3 {
            if board.is_playable(selected.row, selected.col) {
                return;
            }
            match original_col {
                0 => move_selection_right(selected),
                2 => move_selection_left(selected),
                _ => move_selection_right(selected),
            }
        }
    }
}

/// Resets the selected position to the first available cell.
pub fn reset_position(board: &impl Board, selected: &mut Position) {
    (selected.row, selected.col) = (0, 0);
    for _ in 0..9 {
        if board.is_playable(selected.row, selected.col) {
            return;
        }
        move_selection_next(selected);
    }
}

fn move_selection_left(selected: &mut Position) {
    if selected.col > 0 {
        selected.col -= 1;
    } else {
        selected.col = 2;
    }
}

fn move_selection_right(selected: &mut Position) {
    selected.col = (selected.col + 1) % 3;
}

fn move_selection_up(selected: &mut Position) {
    if selected.row > 0 {
        selected.row -= 1;
    } else {
        selected.row = 2;
    }
}

fn move_selection_down(selected: &mut Position) {
    selected.row = (selected.row + 1) % 3;
}

fn move_selection_next(selected: &mut Position) {
    selected.col += 1;
    if selected.col >= 3 {
        selected.col = 0;
        selected.row += 1;
        if selected.row >= 3 {
            selected.row = 0;
        }
    }
}
