use chess_player::do_move;

use chess::{Board, Color, Square, ChessMove, BoardStatus, Game, BitBoard};
use dialoguer::Input;
use dialoguer::theme::ColorfulTheme;
use std::process::exit;
use std::sync::Arc;

mod gameboard;

fn main() {
    println!("Hello, world!");

    let board = Box::new(Board::default());
    println!("{}", board.piece_on(Square::C1).unwrap());
    game_loop(board);
}

//Runs update once and then enters a loop checking if the input is q.
fn game_loop(board: Box<Board>) {
    //Game is only used to check if there should be a draw.
    let mut game = Game::new_with_board(*board.clone());

    //Boards is a history of BitBoards.
    let mut boards = vec![];
    //Result is a touple of the user input and board after the next move.
    let mut result = (String::new(), board.clone());
    //Counts the number of moves for debugging purposes.
    let mut count = 1;

    //draw the gameboard on screen.
    gameboard::draw(board.clone());

    //Run the loop once so the user can quit on the first prompt.
    result = update(board.clone(), Arc::new(boards.clone()), &mut game);
    boards.push(*result.1.color_combined(board.side_to_move()));

    while result.0 != String::from("Q") {
        result = update(result.1.clone(), Arc::new(boards.clone()), &mut game);
        boards.push(*result.1.color_combined(board.side_to_move()));
        count += 1;
        println!("{}", count);
    }
}

//Gets the user or ai input and moves the pieces.
fn update(board: Box<Board>, boards: Arc<Vec<BitBoard>>, game: &mut Game) -> (String, Box<Board>) {
    if game.can_declare_draw() {
        // game.declare_draw();
        println!("Draw");
        // exit(0);
    }
    let status = match board.status() {
        BoardStatus::Checkmate => "Checkmate",
        BoardStatus::Stalemate => "Stalemate",
        _ => "Ongoing"
    };
    println!("{}", status);

    // get_input(board.clone(), "");

    let piece = String::new();

    //Uncomment for user input on White.
    // if board.side_to_move() == Color::White {
    //     let piece: String = get_input(board.clone(),"Select a piece");
    //     let to = get_input(board.clone(), "Select a space to move to");
    //
    //     let result = match make_move(&piece, &to, board.clone(), game) {
    //         Some(r) => Box::new(r),
    //         None => return (piece, board)
    //     };
    //
    //     gameboard::draw(result.clone());
    //     return (piece, result.into());
    // } else {
        let result = Box::new(make_ai_move(do_move(board.clone(), board.side_to_move(), boards).unwrap_or_default(), board, game));
        gameboard::draw(result.clone());

        return (piece, result.into());
    // }

    // gameboard::draw(result.clone());
    // return (piece, result.into());
}

//Get human input with prompt.
fn get_input(board: Box<Board>, prompt: &str) -> String {
    Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{} {}",prompt, match board.side_to_move() {
            Color::Black => "Black",
            _ => "White"
        }))
        .default(".".into())
        .interact_text().unwrap()
        .to_uppercase()
}

//Move a piece based on user input.
fn make_move(piece: &String, to: &String, board: Box<Board>, game: &mut Game) -> Option<Board> {
    let piece_square = Square::from_string(piece.to_lowercase());
    let to_square = Square::from_string(to.to_lowercase());
    if (piece_square == None) || (to_square == None) {
        return None
    }
    let m = ChessMove::new(piece_square.unwrap_or_default(),to_square.unwrap_or_default(), None);
    if !board.legal(m) {
        println!("Illegal move!");
        return None
    }
    let mut result = Board::default();
    board.make_move(m, &mut result);
    game.make_move(m);
    return Some(result)
}

//Move a piece based on ai input.
fn make_ai_move(m: ChessMove, board: Box<Board>, game: &mut Game) -> Board {
    let mut result = Board::default();
    board.make_move(m, &mut result);
    game.make_move(m);
    return result
}
