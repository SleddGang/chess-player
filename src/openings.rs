use chess::{Board, ChessMove, Color, MoveGen, Piece, BoardStatus, BitBoard, Square};
use std::sync::Arc;

//shut up alexc

pub fn openings(board: Box<Board> , previous_boards: Arc<Vec<BitBoard>>) -> Option<ChessMove>  {
    if previous_boards.len() <= 0 {
        Some(ChessMove::new(Square::D2, Square::D4, None))
    } else if previous_boards.len() == 2 {
        Some(ChessMove::new(Square::G1, Square::F3, None))
    } else { 
        None
    }
}