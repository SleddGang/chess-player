use chess::{Board, ChessMove, Color, Square, MoveGen, Piece, BoardStatus};
use threadpool::ThreadPool;
use std::sync::mpsc::channel;

pub struct Outcomes {
    pub draw: f64,
    pub my_checkmate: f64,
    pub my_check: f64,
    pub their_checkmate: f64,
    pub their_check: f64,
    pub queen: f64,
    pub knight: f64,
    pub bishop: f64,
    pub rook: f64,
    pub pawn: f64,
}

static OUTCOMES: Outcomes = Outcomes {
    draw: 500.0,
    my_checkmate: 1000.0,
    my_check: 100.0,
    their_checkmate: -1000.0,
    their_check: -100.0,
    queen: 100.0,
    knight: 50.0,
    bishop: 50.0,
    rook: 50.0,
    pawn: 1.0
};

const NUMTHREADS: usize = 8;
const MAXDEPTH: isize = 5;
const OUTCOMESMULTIPLIER: f64 = 2.5;
const NEXTMOVEMULTIPLIER: f64 = 2.0;
const TAKEPIECEMULTIPLIER: f64 = 1.01;


pub fn do_move(board: Box<Board>, color: Color) -> Option<ChessMove> {
    if board.side_to_move() != color {
        return None
    }



    let it = MoveGen::new_legal(&board);
    // let mut moves: Vec<(ChessMove, isize, isize)> = vec![(ChessMove::new(Square::A1, Square::A1, None), 0, 0); it.len()];
    let length = it.len();
    let mut moves: Vec<(ChessMove, f64, isize)> = vec![];
    let pool = ThreadPool::new(NUMTHREADS);

    let (tx, rx) = channel();
    for i in it {
        let tx = tx.clone();
        let board = board.clone();
        pool.execute(move|| {
            let target = board.piece_on(i.get_dest());
            let mut result = Board::from(*board.clone());
            board.make_move(i, &mut result);

            let mut local_score: f64 = 0.0;

            match target {
                Some(t) => {
                    if color == board.side_to_move() {
                        local_score += match_piece(t) * OUTCOMESMULTIPLIER * NEXTMOVEMULTIPLIER * TAKEPIECEMULTIPLIER;
                    } else {
                        local_score -= match_piece(t) * OUTCOMESMULTIPLIER * NEXTMOVEMULTIPLIER;
                    }
                },
                None => {},
            }
            match board.status() {
                BoardStatus::Checkmate => {
                    if board.side_to_move() == color {
                        local_score += OUTCOMES.my_checkmate * OUTCOMESMULTIPLIER * NEXTMOVEMULTIPLIER;
                    } else {
                        local_score += OUTCOMES.their_checkmate * OUTCOMESMULTIPLIER * NEXTMOVEMULTIPLIER;
                    }
                },
                _ => {}
            };

            match check_moves(Box::new(result), color, 1) {
                Some((m, s, d)) => {
                    if color == board.side_to_move() {
                        local_score += s;
                    }
                    // moves.push((i, local_score, d));
                },
                None => {
                    // moves.push((i, local_score, 0));
                },
            };
            tx.send((i, local_score, 0));
        });
    }
    for i in rx.iter().take(length) {
        moves.push(i);
    }
    if moves.len() > 0 {
        let mut max: f64 = moves.get(0).unwrap().1;
        let mut selected_move = moves.get(0).unwrap().0;
        for i in moves {
            if i.1 > max {
                selected_move = i.0;
                max = i.1;
            }
        }
        Some(selected_move)
    } else { None }

    // return Some(ChessMove::new(Square::A1, Square::A2, None))
}

fn check_moves(board: Box<Board>, color: Color, mut depth: isize) -> Option<(ChessMove, f64, isize)> {
    depth += 1;
    if depth > MAXDEPTH {
        return None;
    }

    let it = MoveGen::new_legal(&board);
    // let mut moves: Vec<(ChessMove, isize, isize)> = vec![(ChessMove::new(Square::A1, Square::A1, None), 0, 0); it.len()];
    let mut moves: Vec<(ChessMove, f64, isize)> = vec![];
    let max_depth = depth;
    let mut local_score: f64 = 0.0;

    for i in it {
        let target = board.piece_on(i.get_dest());
        let mut result = Board::from(*board.clone());
        board.make_move(i, &mut result);

        // let mut local_score: f64 = 0.0;


        match target {
            Some(t) => {
                if color == board.side_to_move() {
                    local_score += match_piece(t) / depth as f64 * OUTCOMESMULTIPLIER;
                } else {
                    local_score -= match_piece(t) / depth as f64 * OUTCOMESMULTIPLIER;
                }
            },
            None => {},
        }
        match board.status() {
            BoardStatus::Checkmate => {
                if board.side_to_move() == color {
                    local_score += OUTCOMES.my_checkmate / depth as f64 * OUTCOMESMULTIPLIER;
                } else {
                    local_score += OUTCOMES.their_checkmate / depth as f64 * OUTCOMESMULTIPLIER;
                }
            },
            _ => {}
        };

        match check_moves(Box::new(result), color, depth) {
            Some((m, s, d)) => {
                if color == board.side_to_move() {
                    local_score += s;
                }
                moves.push((i, local_score, d));
            },
            None => {
                moves.push((i, local_score, 0));
            },
        };
    }
    if moves.len() > 0 {
        let mut max: f64 = moves.get(0).unwrap().1;
        let mut selected_move = moves.get(0).unwrap().0;
        for i in moves {
            if i.1 > max {
                selected_move = i.0;
                max = i.1;
            }
        }

        Some((selected_move, local_score, max_depth))
    } else { None }
}

fn match_piece(piece: Piece) -> f64 {
    match piece {
        Piece::Rook => OUTCOMES.rook,
        Piece::Queen => OUTCOMES.queen,
        Piece::Pawn => OUTCOMES.pawn,
        Piece::Knight => OUTCOMES.knight,
        Piece::Bishop => OUTCOMES.bishop,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use crate::do_move;
    use chess::{Board, Color};

    #[test]
    fn it_works() {
        do_move(Box::new(Board::default()), Color::White);
        assert_eq!(2 + 2, 4);
    }
}
