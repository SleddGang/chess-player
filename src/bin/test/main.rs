use chess_player::do_move;

use chess::{Board, Color, Square, ChessMove, BoardStatus};
use dialoguer::Input;
use dialoguer::theme::ColorfulTheme;

mod gameboard;

fn main() {
    println!("Hello, world!");

    let board = Box::new(Board::default());
    println!("{}", board.piece_on(Square::C1).unwrap());
    // gameboard::draw(board);
    // make_move(String::from("A2"), String::from("A3"), board);
    game_loop(board);
}

fn game_loop(board: Box<Board>) {
    gameboard::draw(board.clone());
    let mut result = update(board.clone());
    while result.0 != String::from("Q") {
        result = update(result.1.clone());
    }
}

fn update(board: Box<Board>) -> (String, Box<Board>) {
    let status = match board.status() {
        BoardStatus::Checkmate => "Checkmate",
        BoardStatus::Stalemate => "Stalemate",
        _ => "Ongoing"
    };
    println!("{}", status);

    // get_input(board.clone(), "");

    let piece = String::new();
    // if board.side_to_move() == Color::White {
    //     let piece: String = get_input(board.clone(),"Select a piece");
    //     let to = get_input(board.clone(), "Select a space to move to");
    //
    //     let result = match make_move(&piece, &to, board.clone()) {
    //         Some(r) => Box::new(r),
    //         None => return (piece, board)
    //     };
    //
    //     gameboard::draw(result.clone());
    //     return (piece, result.into());
    // } else {
        let result = Box::new(make_ai_move(do_move(board.clone(), board.side_to_move()).unwrap_or_default(), board));
        gameboard::draw(result.clone());
        return (piece, result.into());
    // }

    // gameboard::draw(result.clone());
    // return (piece, result.into());
}

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

fn make_move(piece: &String, to: &String, board: Box<Board>) -> Option<Board> {
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
    return Some(result)
}

fn make_ai_move(m: ChessMove, board: Box<Board>) -> Board {
    let mut result = Board::default();
    board.make_move(m, &mut result);
    return result
}

fn is_valid(input: &String) {

}
