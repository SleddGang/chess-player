use chess::{Board, Square, Rank, File, Piece, Color};

//Prints the letters and then calls draw_horizontal to draw the rest of the board.
pub fn draw(baord: Box<Board>) {
    println!("   A   B   C   D   E   F   G   H   ");
    for i in 1..18 {
        draw_horizontal(i, baord.clone());
    }
}

//Print a horizontal row.  Chooses what to print based on the index.
fn draw_horizontal(count: i32, board: Box<Board>) {
    if count == 1 {
        println!(" ┌───┬───┬───┬───┬───┬───┬───┬───┐");
    } else if count == 17 {
        println!(" └───┴───┴───┴───┴───┴───┴───┴───┘");
    } else if count % 2 == 0 {
        let mut squares = Vec::new();
        for i in 0..=7 {
            squares.push(get_square(((count / 2) - 1) as usize, i));
        }
        print!("{}│", count / 2);
        for s in squares {
            print!(" {} │", draw_piece(board.piece_on(s), board.color_on(s).unwrap_or(Color::Black)) )
        }
        println!()
    } else {
        println!(" ├───┼───┼───┼───┼───┼───┼───┼───┤");
    }
}

//Returns a Square from a rank and file.
fn get_square(rank: usize, file: usize) -> Square {
    Square::make_square(Rank::from_index(rank), File::from_index(file))
}

//Converts a piece and color into a unicode character.
fn draw_piece(piece: Option<Piece>, color: Color) -> String {
    match piece {
        Some(p) => {
            match color {
                Color::Black => {
                    match p {
                        Piece::Bishop => String::from("♝"),
                        Piece::King => String::from("♚"),
                        Piece::Knight => String::from("♞"),
                        Piece::Pawn => String::from("♟"),
                        Piece::Queen => String::from("♛"),
                        Piece::Rook => String::from("♜"),
                    }
                }
                _ => {
                    match p {
                        Piece::Bishop => String::from("♗"),
                        Piece::King => String::from("♔"),
                        Piece::Knight => String::from("♘"),
                        Piece::Pawn => String::from("♙"),
                        Piece::Queen => String::from("♕"),
                        Piece::Rook => String::from("♖"),
                    }
                }
            }
        },
        None => String::from(" ")
    }
}