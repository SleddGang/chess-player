use chess::{Board, Square, Rank, File, Piece, Color};

pub fn draw(baord: Box<Board>) {
    println!("   A   B   C   D   E   F   G   H   ");
    for i in 1..18 {
        draw_horizontal(i, baord.clone());
    }
}

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
        // println!("{}│ {} │ {} │ {} │ {} │ {} │ {} │ {} │ {} │", count / 2,
        //     draw_piece(board.piece_on(Square::make_square(Rank::from_index(0), File::from_index((count / 2) as usize))), board.color_on()),
        //
        // );
        print!("{}│", count / 2);
        for s in squares {
            print!(" {} │", draw_piece(board.piece_on(s), board.color_on(s).unwrap_or(Color::Black)) )
        }
        println!()
    } else {
        println!(" ├───┼───┼───┼───┼───┼───┼───┼───┤");
    }
}

fn get_square(rank: usize, file: usize) -> Square {
    Square::make_square(Rank::from_index(rank), File::from_index(file))
}

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