mod game;

use game::GameBoard;

fn main() { 
    let mut board = GameBoard::new(3);
    board.play_game();
}
