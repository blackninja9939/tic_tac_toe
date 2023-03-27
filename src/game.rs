use std::{fmt, num::ParseIntError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Coordinate {
    x: usize,
    y: usize,
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PositionState {
    Nought,
    Cross,
}

impl fmt::Display for PositionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PositionState::Cross => write!(f, "X"),
            PositionState::Nought => write!(f, "O"),
        }
    }
}

enum GameResult {
    Ongoing,
    Draw,
    NoughtWin,
    CrossWin,
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameResult::Ongoing => write!(f, "Ongoing"),
            GameResult::Draw => write!(f, "Draw!"),
            GameResult::NoughtWin => write!(f, "Noguhts win!"),
            GameResult::CrossWin => write!(f, "Crosses win!"),
        }
    }
}

impl From<PositionState> for GameResult {
    fn from(state: PositionState) -> Self {
        match state {
            PositionState::Cross => GameResult::CrossWin,
            PositionState::Nought => GameResult::NoughtWin,
        }
    }
}

#[derive(Debug)]
enum MoveError {
    InvalidCoordinate(Coordinate),
    InvalidMove(PositionState, PositionState),
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MoveError::InvalidCoordinate(coord) => {
                write!(f, "{} is an invalid coordinate", coord)
            }
            MoveError::InvalidMove(to, from) => write!(f, "Cannot move from {} to {}", from, to),
        }
    }
}

#[derive(Debug)]
enum ParseMoveError {
    FormatError,
    CoordinateError(ParseIntError),
}

impl fmt::Display for ParseMoveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseMoveError::FormatError => {
                write!(f, "Invalid format, should be x,y,M where M is X or O")
            }
            ParseMoveError::CoordinateError(e) => write!(f, "Invalid coordinate due to {}", e),
        }
    }
}

impl From<ParseIntError> for ParseMoveError {
    fn from(e: ParseIntError) -> Self {
        Self::CoordinateError(e)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParsedMove {
    Quit,
    Move(Coordinate),
}

fn parse_move(input: &str) -> Result<ParsedMove, ParseMoveError> {
    if input == "q" || input == "Q" {
        return Ok(ParsedMove::Quit);
    }

    let args = input.split(',').collect::<Vec<_>>();
    if args.len() != 2 {
        return Err(ParseMoveError::FormatError);
    }

    let x = args[0].parse::<usize>()?;
    let y = args[1].parse::<usize>()?;
    Ok(ParsedMove::Move(Coordinate { x, y }))
}

mod input {
    use std::io;

    pub trait GameInputReader {
        fn read(&mut self) -> Option<String>;
    }

    pub struct StdInGameReader;

    impl StdInGameReader {
        pub fn new() -> StdInGameReader {
            StdInGameReader {}
        }
    }

    impl GameInputReader for StdInGameReader {
        fn read(&mut self) -> Option<String> {
            let mut input = String::new();
            let read_result = io::stdin().read_line(&mut input);
            match read_result {
                Ok(_) => Some(input),
                Err(_) => None,
            }
        }
    }

    pub struct PresetMoveReader {
        moves: Vec<String>,
        index: usize,
    }

    impl PresetMoveReader {
        #[allow(dead_code)] // Used in test and exposed publicly for other users too
        pub fn new<T: AsRef<str>>(moves: &[T]) -> PresetMoveReader {
            PresetMoveReader {
                moves: moves.iter().map(|s| s.as_ref().to_string()).collect(),
                index: 0,
            }
        }
    }

    impl GameInputReader for PresetMoveReader {
        fn read(&mut self) -> Option<String> {
            if self.index >= self.moves.len() {
                return None;
            }
            let val = self.moves[self.index].clone();
            self.index += 1;
            Some(val)
        }
    }
}

// todo[mc] make moves be applied by an entry that we record so we can have undo and redo options
// struct MoveEntry
// {
//     position: Coordinate,
//     state: PositionState,
// }

// struct MoveEntryRecord
// {
//     entry: MoveEntry,
//     initial_state: PositionState,
// }

pub struct GameBoard {
    dimension: usize,
    data: Vec<Option<PositionState>>,
    moves_made: usize,
    max_moves: usize,
}

impl GameBoard {
    pub fn new(dimension: usize) -> GameBoard {
        GameBoard {
            dimension,
            data: vec![None; dimension * dimension],
            moves_made: 0,
            max_moves: dimension.pow(2) - 1,
        }
    }

    fn valid_coordinate(&self, pos: Coordinate) -> bool {
        pos.x < self.dimension && pos.y < self.dimension
    }

    fn to_index(&self, pos: Coordinate) -> usize {
        pos.x + (pos.y * self.dimension)
    }

    fn determine_line_result<T: Fn(usize) -> Coordinate>(
        &self,
        state: PositionState,
        coord_func: T,
    ) -> Option<GameResult> {
        for i in 0..self.dimension {
            let coord = coord_func(i);
            let entry = self.data[self.to_index(coord)];

            match entry {
                Some(s) => {
                    if s != state {
                        break;
                    }
                }
                None => break,
            }

            if i == self.dimension - 1 {
                return Some(state.into());
            }
        }
        None
    }

    fn determine_game_result(&self, pos: Coordinate, state: PositionState) -> GameResult {
        // Check columns
        if let Some(result) = self.determine_line_result(state, |y| Coordinate { x: pos.x, y }) {
            return result;
        }

        // Check rows
        if let Some(result) = self.determine_line_result(state, |x| Coordinate { x, y: pos.y }) {
            return result;
        }

        // Check diagonal
        if pos.x == pos.y {
            if let Some(result) = self.determine_line_result(state, |i| Coordinate { x: i, y: i }) {
                return result;
            }
        }

        // Check opposite diagonal
        if pos.x + pos.y == self.dimension - 1 {
            if let Some(result) = self.determine_line_result(state, |i| Coordinate {
                x: i,
                y: self.dimension - 1 - i,
            }) {
                return result;
            }
        }

        if self.moves_made == self.max_moves {
            return GameResult::Draw;
        }

        GameResult::Ongoing
    }

    fn make_move(
        &mut self,
        pos: Coordinate,
        new_state: PositionState,
    ) -> Result<GameResult, MoveError> {
        if !self.valid_coordinate(pos) {
            return Err(MoveError::InvalidCoordinate(pos));
        }

        let index = self.to_index(pos);

        let entry = &self.data[index];
        if let Some(state) = entry {
            return Err(MoveError::InvalidMove(new_state, *state));
        }

        let entry = &mut self.data[index];
        *entry = Some(new_state);

        self.moves_made += 1;

        Ok(self.determine_game_result(pos, new_state))
    }

    fn print(&self) {
        let mut to_print = String::with_capacity(self.dimension);
        for y in 0..self.dimension {
            to_print.clear();

            for x in 0..self.dimension {
                let coord = Coordinate { x, y };
                let entry = self.data[self.to_index(coord)];
                match entry {
                    Some(state) => to_print += &state.to_string(),
                    None => to_print += " ",
                }
            }

            println!("{}", to_print);
        }
    }

    pub fn play_game(&mut self) {
        let input_reader = input::StdInGameReader::new();
        self.play_game_with_reader(input_reader);
    }

    pub fn play_game_with_reader<T: input::GameInputReader>(&mut self, mut input_reader: T) {
        println!("Lets play tic tac toe!");

        let mut current_side = PositionState::Nought;

        loop {
            println!(
                "{} play, enter x,y coordinate to pick tile or Q to quit!",
                current_side
            );

            let input = match input_reader.read() {
                Some(input) => input,
                None => {
                    println!("Failed to read input");
                    break;
                }
            };

            let parsed_move = match parse_move(input.trim()) {
                Ok(parse_move) => parse_move,
                Err(bad_move) => {
                    println!("{}", bad_move);
                    continue;
                }
            };

            let move_result = match parsed_move {
                ParsedMove::Quit => {
                    println!("Quitting!");
                    break;
                }
                ParsedMove::Move(move_pos) => {
                    let move_result = self.make_move(move_pos, current_side);
                    self.print();
                    move_result
                }
            };

            match move_result {
                Ok(GameResult::Ongoing) => {
                    current_side = match current_side {
                        PositionState::Cross => PositionState::Nought,
                        PositionState::Nought => PositionState::Cross,
                    };
                }
                Ok(game_result) => {
                    println!("{}", game_result);
                    break;
                }
                Err(move_error) => {
                    println!("{}, try again", move_error);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_err {
        ($expression:expr, $($pattern:tt)+) => {
            match $expression {
                $($pattern)+ => (),
                ref e => panic!("expected `{}` but got `{:?}`", stringify!($($pattern)+), e),
            }
        }
    }

    #[test]
    fn parse_move_test() {
        assert_err!(
            parse_move("3,6"),
            Ok(ParsedMove::Move(Coordinate { x: 3, y: 6 }))
        );

        assert_err!(parse_move("q"), Ok(ParsedMove::Quit));
        assert_err!(parse_move("Q"), Ok(ParsedMove::Quit));

        assert_err!(parse_move("1,2,3"), Err(ParseMoveError::FormatError));

        // We safely assume that string to int parsing returns the right errors, so instead of checking specific error cases just do some broad checks
        assert!(parse_move("a,1").is_err());
        assert!(parse_move(",").is_err());
        assert!(parse_move("-1,0").is_err());
    }

    #[test]
    fn test_game() {
        let moves = ["0,0", "1,1", "0,1", "2,0", "0,2"];
        let input_reader = input::PresetMoveReader::new(&moves);
        let mut game_board = GameBoard::new(3);
        game_board.play_game_with_reader(input_reader);
    }
}
