#![feature(test)]

use chess::{Board, ChessMove, Color, MoveGen, Piece, BoardStatus, BitBoard};
use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use std::sync::Arc;

//Outcomes is a struct to organize all the possible point values.
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

//Initialize a static Outcomes to with values.
static OUTCOMES: Outcomes = Outcomes {
    draw: 500.0,
    my_checkmate: 1000.0,
    my_check: 100.0,
    their_checkmate: 1000.0,
    their_check: 100.0,
    queen: 100.0,
    knight: 50.0,
    bishop: 50.0,
    rook: 50.0,
    pawn: 1.0
};

const NUMTHREADS: usize = 8;            //NUMTHREADS is the number of threads to run the ai on.
const MAXDEPTH: isize = 4;              //The max number of moves run.
const OUTCOMESMULTIPLIER: f64 = 2.5;    //Number to multiply the outcomes by.
// const NEXTMOVEMULTIPLIER: f64 = 2.0; //A higher number increases the points that
const TAKEPIECEMULTIPLIER: f64 = 1.01;  //Applied when taking a piece not when losing a piece.

//Main entry point for the ai.  Board is the current board state, color is the color of the side the bot is on, and previous_boards is a vector of all the previous
//BitBoards and is used to enforce the threefold rule in the ai.
pub fn do_move(board: Box<Board>, color: Color, previous_boards: Arc<Vec<BitBoard>>) -> Option<ChessMove> {
    //Check to make sure the ai is moving next.
    if board.side_to_move() != color {
        return None
    }

    let it = MoveGen::new_legal(&board);                     //Get a iterator of next legal moves.
    let length = it.len();                                  //Size of the iterator.  Used later to set an endpoint for the rx.
    let mut moves: Vec<(ChessMove, f64, isize)> = vec![];             //Initialize a vector of empty moves and points that the move could lead to.
                                                                            //The final move will be selected from this vector.
    let pool = ThreadPool::new(NUMTHREADS);     //Thread pool which will run the ai.

    //Thread safe message passing.  Used to pass out the moves and scores out of the thread.
    let (tx, rx) = channel();
    //Loop through each possible move.
    for i in it {
        let tx = tx.clone();
        let board = board.clone();
        let previous_boards:Arc<Vec<BitBoard>> = Arc::clone(&previous_boards);

        pool.execute(move|| {
            let target = board.piece_on(i.get_dest());  //Gets the target piece of the move.  None if there is no target piece.

            //Make a the move and put the board in result.
            let mut result = Board::from(*board.clone());
            board.make_move(i, &mut result);

            let mut score = 0.0;

            match target {
                Some(t) => {
                    score += match_piece(t);
                },
                None => {}
            };
            match board.status() {
                BoardStatus::Checkmate => {
                    score += OUTCOMES.their_checkmate;
                },
                _ => {}
            };

            if !is_threefold(*result.combined(), previous_boards.clone()) {
                match search_min(0.0, 0.0, Box::new(result), 1, score, previous_boards.clone()) {
                    Some(s) => {
                        // println!("Score: {}", s);
                        tx.send(Some((i, s + score, 0)));
                    }
                    None => {
                        tx.send(None);
                    }
                }
            }
        })
    }

    //Match the values sent by the txs into moves if the value sent by tx is not None.
    for i in rx.iter().take(length) {
        match i {
            Some(m) => {
                moves.push(m);
            },
            None => {}
        };
    }

    //If the lenght of moves is zero then return none.  Otherwise find the move with the highest score and return it.
    if moves.len() > 0 {
        let mut max: f64 = moves.get(0).unwrap().1;
        let mut selected_move = moves.get(0).unwrap().0;
        for i in moves {
            if i.1 > max {
                selected_move = i.0;
                max = i.1;
            }
        }
        println!("Max Score: {}", max);
        Some(selected_move)
    } else { None }

    // return Some(ChessMove::new(Square::A1, Square::A2, None))
}

fn search_max(mut alpha: f64, beta: f64, board: Box<Board>, mut depth: isize, total_score: f64, previous_boards: Arc<Vec<BitBoard>>) -> Option<f64> {
    //Add one the the depth.
    depth += 1;

    let it = MoveGen::new_legal(&board);
    let mut scores = vec![];

    for i in it {
        let target = board.piece_on(i.get_dest());  //Gets the target piece of the move.  None if there is no target piece.

        //Make a the move and put the board in result.
        let mut result = Board::from(*board.clone());
        board.make_move(i, &mut result);

        let mut score = 0.0;

        match target {
            Some(t) => {
                score += match_piece(t);
            },
            None => {}
        };

        match result.status() {
            BoardStatus::Checkmate => {
                score += OUTCOMES.their_checkmate;
            },
            _ => {}
        };

        score += total_score;

        if !is_threefold(*result.combined(), previous_boards.clone()) {
            if depth < MAXDEPTH {
                match search_min(alpha, beta, Box::new(result), depth, score, previous_boards.clone()) {
                    Some(s) => {
                        score += s;
                        scores.push(score);
                        // if score >= beta {
                        //     return Some(beta)
                        // } else if score > alpha {
                        //     alpha = score;
                        // }
                    },
                    None => {}
                };
            } else {
                scores.push(score);
            }
        }
    }

    if scores.len() > 0 {
        let mut max = *scores.get(0).expect("Error unwrapping score.");
        for score in scores {
            if score > max {
                max = score;
            }
        }
        return Some(max);
    }
    return None;
}

fn search_min(alpha: f64, mut beta: f64, board: Box<Board>, mut depth: isize, total_score: f64, previous_boards: Arc<Vec<BitBoard>>) -> Option<f64> {
    //Add one the the depth.
    depth += 1;

    let it = MoveGen::new_legal(&board);
    let mut scores = vec![];

    for i in it {
        let target = board.piece_on(i.get_dest());  //Gets the target piece of the move.  None if there is no target piece.

        //Make a the move and put the board in result.
        let mut result = Board::from(*board.clone());
        board.make_move(i, &mut result);

        let mut score = 0.0;

        match target {
            Some(t) => {
                score -= match_piece(t);
            },
            None => {}
        };
        match result.status() {
            BoardStatus::Checkmate => {
                score -= OUTCOMES.my_checkmate;
            },
            _ => {}
        };

        score += total_score;

        if !is_threefold(*result.combined(), previous_boards.clone()) {
            if depth < MAXDEPTH {
                match search_max(alpha, beta, Box::new(result), depth, score, previous_boards.clone()) {
                    Some(s) => {
                        score += s;
                        scores.push(score);
                        // if score <= alpha {
                        //     return Some(alpha)
                        // } else if score < beta {
                        //     beta = score;
                        // }
                    },
                    None => {}
                };
            } else {
                scores.push(score);
            }
        }
    }

    if scores.len() > 0 {
        let mut min = *scores.get(0).expect("Error unwrapping score.");
        for score in scores {
            if score < min {
                min = score;
            }
        }
        return Some(min);
    }
    return None;
}

//Check to make see if the BitBoard of board is found in boards more than once.
//Returns true if the threefold criteria is met.  Otherwise returns false.
fn is_threefold(board: BitBoard, boards: Arc<Vec<BitBoard>>) -> bool {
    if boards.iter().filter(|&b| *b == board).count() >= 2 {
        // println!("Threefold");
        return true;
    }
    return false;
}

//Match a Chess Piece to a score.
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
    extern crate test;

    use crate::do_move;
    use chess::{Board, Color, Square};
    use self::test::Bencher;
    use std::sync::Arc;

    //Tests a single move from the default starting position.  Dosn't really work.....
    #[test]
    fn it_works() {
        let m = do_move(Box::new(Board::default()), Color::White, Arc::new(vec![])).unwrap();
        println!("{} to {}", m.get_source(), m.get_dest());
        assert_eq!(m.get_dest(), Square::A3);
        assert_eq!(m.get_source(), Square::A2);
    }

    //Benchmark a single move from the default starting position.
    #[bench]
    fn bench_default(b: &mut Bencher) {
        let board = Box::new(Board::default());
        b.iter(move || {
            do_move(board.clone(), Color::White, Arc::new(vec![]));
        })
    }
}
