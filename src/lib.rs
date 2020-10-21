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
    their_checkmate: -1000.0,
    their_check: -100.0,
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
        //Clone variables needed in the thread.
        let tx = tx.clone();
        let board = board.clone();
        let previous_boards:Arc<Vec<BitBoard>> = Arc::clone(&previous_boards);
        //Start the thread.
        pool.execute(move|| {
            //get the target piece of the move.  If the move is not going take a piece the target will be None
            let target = board.piece_on(i.get_dest());
            //Make the move i and put the resulting board in result.
            let mut result = Board::from(*board.clone());
            board.make_move(i, &mut result);

            //Current score of the move.  The score of this move and the scores of consecutive moves get added to local_score.
            let mut local_score: f64 = 0.0;

            //Check if target is something and if the move involes the ai's color taking a piece then and the points to local_score.
            //Otherwise if it is not the ai's move then subtract the points from local_score.
            //If there is no piece being taken do nothing.
            match target {
                Some(t) => {
                    if color == board.side_to_move() {
                        local_score += match_piece(t) * OUTCOMESMULTIPLIER * TAKEPIECEMULTIPLIER;
                    } else {
                        local_score -= match_piece(t) * OUTCOMESMULTIPLIER;
                    }
                },
                None => {},
            }
            //If a move's outcome is checkmate add or subtract the points from local_score depending on who is making the move.
            //If there is not a checkmate do noting.
            match board.status() {
                BoardStatus::Checkmate => {
                    if board.side_to_move() == color {
                        local_score += OUTCOMES.my_checkmate * OUTCOMESMULTIPLIER;
                    } else {
                        local_score += OUTCOMES.their_checkmate * OUTCOMESMULTIPLIER;
                    }
                },
                _ => {}
            };

            //Call check_moves with the result.  This will continue the chain of checking moves.
            //If check_moves returns a score add it to local_score.
            match check_moves(Box::new(result), color, 1, previous_boards.clone()) {
                Some((_, s, _)) => {
                    if color == board.side_to_move() {
                        local_score += s;
                    }
                    // moves.push((i, local_score, d));
                },
                None => {
                    // moves.push((i, local_score, 0));
                },
            };

            //If this move does not break the threefold rule send the move and its score through tx.
            //Otherwise send nothing.
            if !is_threefold(*result.color_combined(color), previous_boards.clone()) {
                tx.send(Some((i, local_score, 0)));
            } else {
                tx.send(None);
            }
        });
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
        Some(selected_move)
    } else { None }

    // return Some(ChessMove::new(Square::A1, Square::A2, None))
}

//Recursively check all moves possible on board and return the best move and the sum of all the scores.
//Board is the current board to check, color is the color of the side the ai is on, depth is how far into the moves the ai is on to make sure the recursion dosn't go
// on forever, boards is the list of previous board positions to enforce the threefold rule in the ai.
fn check_moves(board: Box<Board>, color: Color, mut depth: isize,  boards: Arc<Vec<BitBoard>>) -> Option<(ChessMove, f64, isize)> {
    //Add one the the depth and make sure we aren't to deep.  If depth is over the MAXDEPTH return None.
    depth += 1;
    if depth > MAXDEPTH {
        return None;
    }


    let it = MoveGen::new_legal(&board);               //Get a iterator of next legal moves.
    let mut moves: Vec<(ChessMove, f64, isize)> = vec![];       //Initialize a vector of empty moves and points that the move could lead to.
    let max_depth = depth;                                //Currently not used.  TODO remember why I put this in here.
    let mut local_score: f64 = 0.0;                             //Sum of this boards move scores and consecutive move scores.

    //Iterate over all possible moves.
    for i in it {
        let target = board.piece_on(i.get_dest());  //Gets the target piece of the move.  None if there is no target piece.

        //Make a the move and put the board in result.
        let mut result = Board::from(*board.clone());
        board.make_move(i, &mut result);

        //If there is a target piece then add or subtract the points form local_score depending on which color is making the move.
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

        //If a move's outcome is checkmate add or subtract the points from local_score depending on who is making the move.
        //If there is not a checkmate do noting.
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

        //Recursivly call check_moves and if it returns something add the score to local_score.
        match check_moves(Box::new(result), color, depth, boards.clone()) {
            Some((_, s, _)) => {
                if color == board.side_to_move() {
                    local_score += s;
                }
            },
            None => {
            },
        };

        //If the threefold criteria are not met then push the current move and score onto moves.
        if !is_threefold(*result.color_combined(color), boards.clone()) {
            moves.push((i, local_score, max_depth));
        }
    }

    //Check if moves is empty.  If so return None.
    //Otherwise find the move with the highest score and return it and local_score.
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
