#![feature(test)]

use chess::{Board, ChessMove, Color, MoveGen, Piece, BoardStatus, BitBoard};
use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

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
const MAXDEPTH: isize = 5;              //The max number of moves run.
const OUTCOMESMULTIPLIER: f64 = 2.5;    //Number to multiply the outcomes by.
// const NEXTMOVEMULTIPLIER: f64 = 2.0; //A higher number increases the points that
const TAKEPIECEMULTIPLIER: f64 = 1.01;  //Applied when taking a piece not when losing a piece.

/// Main entry point for the ai.  Board is the current board state,
/// color is the color of the side the bot is on, and previous_boards is a vector of all the previous
/// BitBoards and is used to enforce the threefold rule in the ai.
/// Inside pool.execute() the code is very similar to search_max() however later we may want to add some
/// top level specific code.
pub fn do_move(board: Box<Board>, color: Color, previous_boards: Arc<Vec<BitBoard>>) -> Option<ChessMove> {
    //Check to make sure the ai is moving next.
    if board.side_to_move() != color {
        return None
    }

    let it = MoveGen::new_legal(&board);                     //Get a iterator of next legal moves.
    let length = it.len();                                  //Size of the iterator.  Used later to set an endpoint for the rx.
    let mut moves: Vec<(ChessMove, f64, bool)> = vec![];             //Initialize a vector of empty moves and points that the move could lead to.
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
            let mut is_mate = false;

            match target {
                Some(t) => {
                    score += match_piece(t);
                },
                None => {}
            };
            match result.status() {
                BoardStatus::Checkmate => {
                    score += OUTCOMES.their_checkmate;
                    is_mate = true;
                },
                _ => {}
            };
            let mut their_score = 0.0;
            match search_min(f64::MIN, f64::MAX, Box::new(result), 1, score, previous_boards.clone()) {
                Some(s) => {
                    their_score = s;
                }
                None => {
                }
            };

            tx.send(Some((i, their_score, is_mate)));
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

    //If the length of moves is zero then return none.  Otherwise find the move with the highest score and return it.
    if moves.len() > 0 {
        let mut max: f64 = moves.get(0).unwrap().1;
        let mut selected_move = moves.get(0).unwrap().0;
        for i in moves {
            if i.2 {
                return Some(i.0)
            }
            if i.1 > max {
                selected_move = i.0;
                max = i.1;
            }
        }
        println!("Max Score: {}", max);
        Some(selected_move)
    } else { None }
}

/// Searches all possible moves for a given board and returns the maximum score plus the score of search_min().
/// This function is used to find the highest score possible for any given board assuming the opponent chooses
/// The path that costs the ai the most amount of points.  Effectively it chooses the move with the highest guaranteed score.
/// This function is only called on ai moves.
fn search_max(mut alpha: f64, beta: f64, board: Box<Board>, mut depth: isize, total_score: f64, previous_boards: Arc<Vec<BitBoard>>) -> Option<f64> {
    //Add one the the depth.
    depth += 1;

    let it = MoveGen::new_legal(&board);
    let mut scores = vec![];

    //Loop through each possible move.
    for i in it {
        //Gets the target piece of the move.  None if there is no target piece.
        let target = board.piece_on(i.get_dest());

        //Make a the move and put the board in result.
        let mut result = Board::from(*board.clone());
        board.make_move(i, &mut result);

        let mut score = 0.0;
        let mut is_mate = false;

        //If the target square is a piece then add the taken pieces points to score.
        match target {
            Some(t) => {
                score += match_piece(t);
                is_mate = true;
            },
            None => {}
        };

        //If this move leads to checkmate then add the checkmate points to score.
        match result.status() {
            BoardStatus::Checkmate => {
                score += OUTCOMES.their_checkmate;
            },
            _ => {}
        };

        //Add previous total scores to score.
        score += total_score;

        //Check if this move breaks the threefold rule.
        //Check if we are too deep.  If not then add the result of search_min() to the score and add score to scores.\
        //Otherwise just add score to scores.
        let mut their_score = 0.0;
        if depth < MAXDEPTH {
            // if !is_mate {
                match search_min(alpha, beta, Box::new(result), depth, score, previous_boards.clone()) {
                    Some(s) => {
                        their_score = s;
                    },
                    None => {}
                };
            // }

            //Add the score to the list of scores.
            score += their_score;
            scores.push(score);

            //Perform alpha beta pruning.
            if score >= beta {
                return Some(beta)
            } else if score > alpha {
                 alpha = score;
            }
        } else {
            scores.push(score);
        }
    }

    //Check to make sure we have some scores and if so find the highest score and return it.
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

/// Searches all possible moves for a given board and returns the minimum score plus the score of search_max().
/// This function is used to find the lowest score possible for any given board assuming the ai chooses
/// The path that guarantees the ai the most amount of points.  Effectively it chooses the move with the lowest guaranteed score.
/// This function is only called on opponent moves.
fn search_min(alpha: f64, mut beta: f64, board: Box<Board>, mut depth: isize, total_score: f64, previous_boards: Arc<Vec<BitBoard>>) -> Option<f64> {
    //Add one the the depth.
    depth += 1;

    let it = MoveGen::new_legal(&board);
    let mut scores = vec![];

    //Loop through each possible move.
    for i in it {
        //Gets the target piece of the move.  None if there is no target piece.
        let target = board.piece_on(i.get_dest());

        //Make a the move and put the board in result.
        let mut result = Board::from(*board.clone());
        board.make_move(i, &mut result);

        let mut score = 0.0;
        let mut is_mate = false;

        //If the target square is a piece then subtract the taken pieces points to score.
        match target {
            Some(t) => {
                score -= match_piece(t);
            },
            None => {}
        };

        //If this move leads to checkmate then subtract the checkmate points to score.
        match result.status() {
            BoardStatus::Checkmate => {
                score -= OUTCOMES.my_checkmate;
                is_mate = true;
            },
            _ => {}
        };

        //Add previous total scores to score.
        score += total_score;

        //Check if we are too deep.  If not then add the result of search_min() to the score and add score to scores.\
        //Otherwise just add score to scores.
        let mut their_score = 0.0;
        if depth < MAXDEPTH {
            // if !is_mate {
                match search_max(alpha, beta, Box::new(result), depth, score, previous_boards.clone()) {
                    Some(s) => {
                        their_score = s;
                    },
                    None => {
                        // println!("None")
                    }
                };
            // }

            //Add the score to the list of scores.
            score += their_score;
            scores.push(score);

            //Perform alpha beta pruning.
            if score <= alpha {
                return Some(alpha)
            } else if score < beta {
                beta = score;
            }
        } else {
            scores.push(score);
        }
    }

    //Check to make sure we have some scores and if so find the lowest score and return it.
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

//Maybe used in the future.  Alternative to adding the points as we go along.
// fn evaluate(board: Board, color: Color) -> f64 {
//     let mut my_score = 0.0;
//     let mut their_score = 0.0;
//
//     for square in *board.color_combined(color) {
//         my_score += match_piece(board.piece_on(square).unwrap());
//     }
//     for square in *board.color_combined(!color) {
//         their_score += match_piece(board.piece_on(square).unwrap());
//     }
//
//     my_score - their_score
// }

/// Check to make see if the BitBoard of board is found in boards more than once.
/// Returns true if the threefold criteria is met.  Otherwise returns false.
fn is_threefold(board: BitBoard, boards: Arc<Vec<BitBoard>>) -> bool {
    if boards.iter().filter(|&b| *b == board).count() >= 2 {
        // println!("Threefold");
        return true;
    }
    return false;
}

/// Match a Chess Piece to a score.
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
