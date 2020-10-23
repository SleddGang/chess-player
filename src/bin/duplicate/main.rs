use std::{env, io};
use std::fs::File;
use std::path::Path;
use std::io::BufRead;
use std::fmt::Write as FmtWrite;

fn main() {
    let mut filename = String::new();
    for a in env::args() {
        filename = a;
    }

    let mut boards  = vec![];
    if let Ok(lines) = read_lines(filename) {
        let mut in_board = false;
        let mut count = 0;
        let mut current_board = String::new();
        for line in lines {
            let l = line.unwrap();
            if in_board == false {
                if l == String::from('#') {
                    in_board = true;
                }
            } else if count > 17 {
                count = 0;
                in_board = false;
                boards.push(current_board.clone());
                current_board = String::new();
            } else {
                count += 1;
                writeln!(&mut current_board, "{}", l.as_str());
            }
        }

        let mut duplicates = vec![];
        for board in boards.clone() {
            // if boards.iter().filter(|&b| *b == board).count() >= 3 {
            //     println!("{}", board);
            // }
            let mut count = 0;
            for b in boards.clone() {
                if b.trim() == board.trim() {
                    count += 1;
                }
            }
            if count >= 2 {
                duplicates.push((board, count));
            }
        }
        for board in duplicates {
            println!("{}: {}", board.0, board.1);
        }
    }
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}