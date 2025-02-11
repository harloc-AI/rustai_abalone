use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use super::marble_moves;

/// number of rows and column for every board representation
pub const BOARD_SIZE: usize = 11;
const BOARD_MAXID: usize = BOARD_SIZE - 1;
pub type Board = [[i8; BOARD_SIZE]; BOARD_SIZE];

/// empty board with off-board position to create the hexagonal shape
pub const EMPTY_BOARD: Board = [
    [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    [3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 3],
    [3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 3],
    [3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 3],
    [3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 3],
    [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3],
    [3, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3],
    [3, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3],
    [3, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3],
    [3, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3],
    [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
];

/// typical "Belgian Daisy" starting position used in competitional play
pub const BELGIAN_DAISY: Board = [
    [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    [3, 3, 3, 3, 3, 1, 1, 0, 2, 2, 3],
    [3, 3, 3, 3, 1, 1, 1, 2, 2, 2, 3],
    [3, 3, 3, 0, 1, 1, 0, 2, 2, 0, 3],
    [3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 3],
    [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3],
    [3, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3],
    [3, 0, 2, 2, 0, 1, 1, 0, 3, 3, 3],
    [3, 2, 2, 2, 1, 1, 1, 3, 3, 3, 3],
    [3, 2, 2, 0, 1, 1, 3, 3, 3, 3, 3],
    [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
];

/// coordinate on an Abalone board
#[derive(Copy, Clone, Hash)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

impl Coord {
    /// creates a new instance
    ///
    /// # Arguments
    ///
    /// * `x` - x value of the coordinate
    /// * `y` - y value of the coordinate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustai_abalone::game::Coord;
    /// let x: usize = 3;
    /// let y: usize = 4;
    /// let coordinate = Coord::new(x, y);
    /// ```
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    /// performs a given MarbleMove multiple times
    ///
    /// # Arguments
    ///
    /// * `marb_move` - MarbleMove to be performed multiple times
    /// * `factor` - number of times the move is performed
    ///
    /// # Returns
    ///
    /// * `new_coord` - a new Coord instance with new x- and y-values
    ///
    /// # Examples
    /// 
    /// ```rust
    /// use rustai_abalone::game::{Coord, MarbleMove};
    /// let coord_init = Coord::new(2, 6);
    /// let marble_move = MarbleMove::new(1, -1);
    /// let coord_new = coord_init.multi_move(&marble_move, 3);  // x == 5 and y == 3 for this case
    /// ```
    pub fn multi_move(self, marb_move: &MarbleMove, factor: usize) -> Self {
        Self::new(
            match marb_move.dx {
                1 => self.x + factor,
                0 => self.x,
                -1 => self.x - factor,
                _ => panic!("illegal move created"),
            },
            match marb_move.dy {
                1 => self.y + factor,
                0 => self.y,
                -1 => self.y - factor,
                _ => panic!("illegal move created"),
            },
        )
    }

    /// checks whether the coordinate is in the general possible range for board
    /// so both values x and y are larger than 0 and smaller than 10
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use rustai_abalone::game::{Coord, MarbleMove};
    /// let coord_in = Coord::new(2, 6);
    /// assert!(coord_in.in_board_range());
    /// 
    /// let coord_out = Coord::new(1, 14);
    /// assert!(!coord_out.in_board_range());
    /// ```
    pub fn in_board_range(&self) -> bool {
        if (self.x > 0 || self.x < BOARD_MAXID) && (self.y > 0 || self.y < BOARD_MAXID) {
            return true;
        }
        false
    }
}

impl std::ops::Add<MarbleMove> for Coord {
    type Output = Self;

    /// performs a marble move operation on a coordinate
    ///
    /// # Arguments
    ///
    /// * `rhs` - the marble move to be performed
    ///
    /// # Returns
    /// * `new_coord` - new Coord struct resulting my "moving" the orignal Coord in the given direction
    ///
    /// # Examples
    /// ```rust
    /// use rustai_abalone::game::{Coord, MarbleMove};
    /// let orig_coord = Coord::new(1, 1);
    /// let marb_move = MarbleMove::new(1, 0);
    /// let new_coord = orig_coord + marb_move; // x == 2, y == 1 for this case
    /// ```
    fn add(self, rhs: MarbleMove) -> Self {
        Self::new(
            match rhs.dx {
                1 => self.x + 1,
                0 => self.x,
                -1 => self.x - 1,
                _ => panic!("illegal move created"),
            },
            match rhs.dy {
                1 => self.y + 1,
                0 => self.y,
                -1 => self.y - 1,
                _ => panic!("illegal move created"),
            },
        )
    }
}

impl std::ops::Sub<MarbleMove> for Coord {
    type Output = Self;

    /// performs the opposite of a given marble move operation on a coordinate
    ///
    /// # Arguments
    ///
    /// * `rhs` - the marble move to be performed
    ///
    /// # Returns
    /// * `new_coord` - new Coord struct resulting my "moving" the orignal Coord against the given direction
    ///
    /// # Examples
    /// ```rust
    /// use rustai_abalone::game::{Coord, MarbleMove};
    /// let orig_coord = Coord::new(1, 1);
    /// let marb_move = MarbleMove::new(1, 0);
    /// let new_coord = orig_coord - marb_move; // x == 0, y == 1 for this case
    /// ```
    fn sub(self, rhs: MarbleMove) -> Self {
        Self::new(
            match rhs.dx {
                1 => self.x - 1,
                0 => self.x,
                -1 => self.x + 1,
                _ => panic!("illegal move created"),
            },
            match rhs.dy {
                1 => self.y - 1,
                0 => self.y,
                -1 => self.y + 1,
                _ => panic!("illegal move created"),
            },
        )
    }
}

impl std::ops::Sub<Coord> for Coord {
    type Output = MarbleMove;

    /// calculates a `MarbleMove` by the difference of two coordinates
    /// 
    /// the function assumes that the marbles are neighbors on the hexagonal board.
    /// If this is not the case the resulting marble move will not make sense
    ///
    /// # Arguments
    ///
    /// * `rhs` - the marble move to be performed
    ///
    /// # Returns
    /// * `difference` - the move that is necessary to move form one coordinate to the neighboring one
    ///
    /// # Examples
    /// ```rust
    /// use rustai_abalone::game::{Coord, MarbleMove};
    /// let orig_coord = Coord::new(4, 4);
    /// let new_coord = Coord::new(5, 4);
    /// let marb_move = new_coord - orig_coord; // dx == 1, dy == 0 for this case
    /// ```
    fn sub(self, rhs: Self) -> MarbleMove {
        let dx: i8 = if self.x > rhs.x { 1 } else { if self.x < rhs.x { -1 } else { 0 }};
        let dy: i8 = if self.y > rhs.y { 1 } else { if self.y < rhs.y { -1 } else { 0 }};
        MarbleMove{ dx, dy }
    }
}

impl PartialEq for Coord {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl PartialOrd for Coord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl  Ord for Coord {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.x == other.x {
            return self.y.cmp(&other.y);
        }
        self.x.cmp(&other.x)
    }
    
}

impl Eq for Coord {}

/// stores values for a move operation
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct MarbleMove {
    /// position change in x direction
    pub dx: i8,
    /// position change in y direction
    pub dy: i8,
}

impl MarbleMove {
    /// returns a marble move with the given direction
    ///
    /// # Arguments
    ///
    /// * `dx` - position change in x direction
    /// * `dy` - position change in y direction
    ///
    /// # Examples
    /// ```rust
    /// use rustai_abalone::game::MarbleMove;
    /// let marb_move = MarbleMove::new(1, 0);
    /// ```
    pub fn new(dx: i8, dy: i8) -> Self {
        Self { dx, dy }
    }
}

/// implementation of the Abalone game
pub struct AbaloneGame {
    board: Board,
    black_tomove: bool,
    next_positions: Vec<Board>,
    state_history: Vec<Board>,
    save_history: bool,
    state_memory: std::collections::HashMap<Board, u8>,
    turn_number: usize,
    noloss_turns: usize,
    noloss_moves: usize,
    white_loss: u8,
    black_loss: u8,
    /// value for the game outcome
    /// -1 = black wins | 0 = draw | 1 = white wins | 10 = game did not end
    result: i8,
    game_ended: bool,
}

impl AbaloneGame {
    // initial vector size for storing the game history
    const MAX_SAVE: usize = 140;
    // maximum number of marbles for each side
    const MARBLES_MAX: u8 = 14;
    // number of marbles a player has to lose in order to suffer a defeat
    const LOSS_DEFEAT: u8 = 6;
    // number of moves to be made without the loss of a marble in order to reach a draw
    const NOLOSS_DRAW: usize = 50;
    // number of times a position has to be repeated in order to reach a draw
    const REPS_TO_DRAW: u8 = 3;

    // values for the board
    const EMPTY: i8 = 0;
    const WHITE: i8 = 1;
    const BLACK: i8 = 2;
    const OFF_BOARD: i8 = 3;

    // maximum number of marbles that can be moved in one row
    const MARBLE_ROW: usize = 3;
    // all possible marble move directions for the hexagonal board
    const MOVES: [MarbleMove; 6] = [
        MarbleMove { dx: 1, dy: 0 },
        MarbleMove { dx: 1, dy: -1 },
        MarbleMove { dx: 0, dy: 1 },
        MarbleMove { dx: -1, dy: 0 },
        MarbleMove { dx: -1, dy: 1 },
        MarbleMove { dx: 0, dy: -1 },
    ];
    // corresponding move direction for "broad side" marble row moves
    const ORTHO_MOVES: [[MarbleMove; 2]; 6] = [
        [MarbleMove { dx: 1, dy: -1 }, MarbleMove { dx: 0, dy: 1 }],
        [MarbleMove { dx: 0, dy: 1 }, MarbleMove { dx: -1, dy: 0 }],
        [MarbleMove { dx: -1, dy: 0 }, MarbleMove { dx: -1, dy: 1 }],
        [MarbleMove { dx: -1, dy: 1 }, MarbleMove { dx: 0, dy: -1 }],
        [MarbleMove { dx: 0, dy: -1 }, MarbleMove { dx: 1, dy: 0 }],
        [MarbleMove { dx: 1, dy: 0 }, MarbleMove { dx: 1, dy: -1 }],
    ];

    /// returns a new Abalone game instance
    ///
    /// # Arguments
    ///
    /// * `board` - the starting position of the game. This position can be any valid position
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// let abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// ```
    ///
    /// # Panics
    ///
    /// will panic if the given board is not a valid one
    pub fn new(board: Board) -> Self {
        if !AbaloneGame::validate_board(board) {
            panic!("the board state for initiation is invalid!")
        }
        let mut new_game = Self {
            board,
            black_tomove: true,
            next_positions: Vec::with_capacity(Self::MAX_SAVE),
            state_history: Vec::with_capacity(Self::MAX_SAVE),
            save_history: true,
            state_memory: std::collections::HashMap::with_capacity(150),
            turn_number: 1,
            noloss_turns: 0,
            noloss_moves: 0,
            white_loss: Self::MARBLES_MAX - Self::count_marbles(board, Self::WHITE),
            black_loss: Self::MARBLES_MAX - Self::count_marbles(board, Self::BLACK),
            result: 10,
            game_ended: false,
        };
        new_game.check_game_ended();
        new_game
    }

    /// creates a copy the current AbaloneGame instance for MCTS
    ///
    /// the copy will contain the all necessary state information,
    /// but it will not store the game history. These copies can be used
    /// for simulation within MCTS
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// # let abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// let copy_for_mcts = abalone.mcts_copy();
    /// ```
    pub fn mcts_copy(&self) -> Self {
        Self {
            board: self.board,
            black_tomove: self.black_tomove,
            next_positions: Vec::with_capacity(Self::MAX_SAVE),
            state_history: vec![],
            save_history: false,
            state_memory: self.state_memory.clone(),
            turn_number: self.turn_number,
            noloss_turns: self.noloss_turns,
            noloss_moves: self.noloss_moves,
            white_loss: self.white_loss,
            black_loss: self.black_loss,
            result: self.result,
            game_ended: self.game_ended,
        }
    }

    /// returns all possible marble moves for the game implementation
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, Coord};
    /// let abalone_moves = AbaloneGame::get_game_moves();
    /// let init_coord = Coord::new(5, 5);
    /// let mut moving_coord = init_coord.clone()
    /// for marble_move in abalone_moves {
    ///     moving_coord += marble_move;
    /// }
    /// assert_eq!(init_coord, moving_coord)
    /// ```
    pub fn get_game_moves() -> [MarbleMove; 6] {
        Self::MOVES
    }

    /// counts the marbles, empty fields or off-board position on a given board
    ///
    /// # Arguments
    /// * `board` - the board state to be analyzed
    /// * `color_code` - field type that will be counted, 0 for empty, 1 for white, 2 for black, and 3 for off-board
    ///
    /// # Examples
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// let black_count = AbaloneGame::count_marbles(BELGIAN_DAISY, 2);
    /// assert_eq!(black_count, 14);
    /// ```
    pub fn count_marbles(board: Board, color_code: i8) -> u8 {
        let mut count: u8 = 0;
        for row in board {
            for field in row {
                if field == color_code {
                    count += 1;
                }
            }
        }
        count
    }

    /// returns the coordinates for black marbles, white marbles and empty fields
    /// from given position
    /// 
    /// # Returns
    /// * `black_coords` - coordinates of all black marbles
    /// * `white_coords` - coordinates of all white marbles
    /// * `empty_coords` - coordinates of all empty fields
    /// 
    /// # Examples
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// let (blacks, whites, empties) = AbaloneGame::coords_by_type(BELGIAN_DAISY);
    /// assert_eq!(blacks.len(), 14);
    /// assert_eq!(whites.len(), 14);
    /// ```
    pub fn coords_by_type(board: Board) -> (Vec<Coord>, Vec<Coord>, Vec<Coord>) {
        let mut black_coords: Vec<Coord> = Vec::with_capacity(14);
        let mut white_coords: Vec<Coord> = Vec::with_capacity(14);
        // there are 61 fields and before there have to be at least 9 + 8 = 17 marbles on the board
        let mut empty_coords: Vec<Coord> = Vec::with_capacity(44);
        for row in 1..BOARD_MAXID {
            for col in 1..BOARD_MAXID {
                match board[row][col] {
                    Self::BLACK => black_coords.push(Coord::new(row, col)),
                    Self::WHITE => white_coords.push(Coord::new(row, col)),
                    Self::EMPTY => empty_coords.push(Coord::new(row, col)),
                    _ => {}
                }
            }
        }
        (black_coords, white_coords, empty_coords)
    }

    /// returns the coordinates for black marbles, white marbles and empty fields
    /// for the current game state
    /// 
    /// # Returns
    /// * `black_coords` - coordinates of all black marbles
    /// * `white_coords` - coordinates of all white marbles
    /// * `empty_coords` - coordinates of all empty fields
    /// 
    /// # Examples
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// let (blacks, whites, empties) = abalone.get_coords_by_type();
    /// assert_eq!(blacks.len(), 14);
    /// assert_eq!(whites.len(), 14);
    /// ```
    pub fn get_coords_by_type(&self) -> (Vec<Coord>, Vec<Coord>, Vec<Coord>) {
        Self::coords_by_type(self.board)
    }

    /// checks whether the given board is a valid Abalone position
    ///
    /// for the position to be valid it must:
    /// * have the same "off-board" edge as the `EMPTY_BOARD`
    /// * no marbles of any color are allowed to exceed `AbaloneGame::MARBLES_MAX`
    /// * no board array value is allowed to be larger than 3
    ///
    /// # Arguments
    ///
    /// * `board` - the board to be checked
    ///
    /// # Examples
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// let is_valid = AbaloneGame::validate_board(BELGIAN_DAISY);
    /// ```
    pub fn validate_board(board: Board) -> bool {
        let mut w_count: u8 = 0;
        let mut b_count: u8 = 0;
        for r in 0..11usize {
            for c in 0..11usize {
                if EMPTY_BOARD[r][c] == 3 && board[r][c] != 3 {
                    return false;
                } else if EMPTY_BOARD[r][c] == 0 {
                    if board[r][c] > 2 {
                        return false;
                    } else if board[r][c] == 1 {
                        w_count += 1;
                    } else if board[r][c] == 2 {
                        b_count += 1;
                    }
                }
            }
        }
        if w_count > Self::MARBLES_MAX {
            return false;
        }
        if b_count > Self::MARBLES_MAX {
            return false;
        }
        true
    }

    fn check_game_ended(&mut self) {
        if self.white_loss >= Self::LOSS_DEFEAT {
            self.result = -1;
        } else if self.black_loss >= Self::LOSS_DEFEAT {
            self.result = 1;
        } else if self.noloss_turns >= Self::NOLOSS_DRAW
            || *self.state_memory.entry(self.board).or_insert(0) >= Self::REPS_TO_DRAW
        {
            self.result = 0;
        }
        if self.result != 10 {
            self.game_ended = true;
        }
    }

    /// standard getter, checks whether the Abalone game has ended
    ///
    /// # Examples
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// # let abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// let is_finished = abalone.get_game_ended();
    /// ```
    pub fn get_game_ended(&self) -> bool {
        self.game_ended
    }

    /// standard getter, returns the game result value
    ///
    /// the values are:
    /// * 10, the game is still running
    /// * -1, black has won the game
    /// * 0, the game was drawn
    /// * 1, white won the game
    ///
    /// # Examples
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// # let abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// let game_result = abalone.get_game_result();
    /// ```
    pub fn get_game_result(&self) -> i8 {
        self.result
    }

    /// standard getter, returns whether black is to move next
    ///
    /// # Examples
    /// 
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// # let abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// let is_black_to_move = abalone.get_black_tomove();
    /// ```
    pub fn get_black_tomove(&self) -> bool {
        self.black_tomove
    }

    /// standard getter for marbles lost by black and white
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// let abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// let (black_loss, white_loss) = abalone.get_black_white_loss();
    /// assert_eq!(black_loss, white_loss);
    /// assert_eq!(black_loss, 0);
    /// ```
    pub fn get_black_white_loss(&self) -> (u8, u8) {
        (self.black_loss, self.white_loss)
    }

    /// gives coordinates for all positions which are not equal between the given board and the game state
    /// 
    /// # Arguments
    /// 
    /// `board` - board state that is compared with the current game state
    /// `marked` - //TODO
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// let abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// let mut selected = std::collections::HashSet::new();
    /// selected.insert(Coord::new(7, 2));
    /// selected.insert(Coord::new(8, 2));
    /// selected.insert(Coord::new(9, 2));
    /// let new_board = [
    ///     [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    ///     [3, 3, 3, 3, 3, 1, 1, 0, 2, 2, 3],
    ///     [3, 3, 3, 3, 1, 1, 1, 2, 2, 2, 3],
    ///     [3, 3, 3, 0, 1, 1, 0, 2, 2, 0, 3],
    ///     [3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 3],
    ///     [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3],
    ///     [3, 0, 2, 0, 0, 0, 0, 0, 0, 3, 3],
    ///     [3, 0, 2, 2, 0, 1, 1, 0, 3, 3, 3],
    ///     [3, 2, 2, 2, 1, 1, 1, 3, 3, 3, 3],
    ///     [3, 2, 0, 0, 1, 1, 3, 3, 3, 3, 3],
    ///     [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    /// ];
    /// abalone.differences_to_state(new_board, &mut selected);
    /// assert_eq!(selected.len(), 4)
    /// ```
    pub fn differences_to_state(&self, board: Board, marked: &mut HashSet<Coord>) {
        let enemy_color = if self.black_tomove {Self::WHITE} else {Self::BLACK};
        let init_marked = marked.len();
        let mut enemy_moved = false;
        let mut inline_origin = Coord::new(1, 1);
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                if board[x][y] != self.board[x][y] {
                    if self.board[x][y] == enemy_color {
                        enemy_moved = true;
                    }
                    if board[x][y] == Self::EMPTY {
                        inline_origin = Coord::new(x, y);
                    }
                    marked.insert(Coord::new(x, y));
                }
            }
        }
        // only special case 3-vs-2-push
        // if an enemy marble was moved it has to be a inline move
        if enemy_moved && init_marked == 3 {
            for marb_move in Self::MOVES {
                // the only field leaving an empty field behind is the base if the marble line
                let next_pos = inline_origin + marb_move;
                // making the original move from this base will result in an already added position
                if marked.contains(&next_pos) {
                    let color_pos = inline_origin.multi_move(&marb_move, 4);
                    if board[color_pos.x][color_pos.y] == enemy_color {
                        marked.insert(inline_origin.multi_move(&marb_move, 4));
                    }
                    break;
                }
            }
        }
    }

    /// switches the colors of a board state and inverses the marble positions
    ///
    /// # Examples
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// let inversed = AbaloneGame::rotate_board(BELGIAN_DAISY);
    /// ```
    pub fn rotate_board(board: Board) -> Board {
        let mut rotated: Board = [[0; BOARD_SIZE]; BOARD_SIZE];
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                if board[x][y] == Self::WHITE {
                    rotated[BOARD_MAXID - x][BOARD_MAXID - y] = Self::BLACK;
                } else if board[x][y] == Self::BLACK {
                    rotated[BOARD_MAXID - x][BOARD_MAXID - y] = Self::WHITE;
                } else {
                    rotated[BOARD_MAXID - x][BOARD_MAXID - y] = board[x][y];
                }
            }
        }
        rotated
    }

    /// standard get methode to obtain the current game state
    pub fn get_state(&self) -> Board {
        self.board
    }

    /// returns the current board state such that it is seen from white's perspective
    ///
    /// this function is meant to return state such that it is viewed from white's
    /// perspective. This means, that if it is black to move the board will be inverted
    /// and colors changed. This allows the AI to predict a move, as it sees the white
    /// marbles as its own
    ///
    /// # Examples
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// # let abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// let whites_perspective = abalone.get_rotated_state();
    /// ```
    pub fn get_rotated_state(&self) -> Board {
        if self.black_tomove {
            return Self::rotate_board(self.board);
        }
        self.board
    }

    /// returns the current board state where colors for marbles were switched
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// let switched = AbaloneGame::switch_colors(BELGIAN_DAISY);
    /// ```
    pub fn switch_colors(board: Board) -> Board {
        let mut switched: Board = [[0; BOARD_SIZE]; BOARD_SIZE];
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                if board[x][y] == Self::WHITE {
                    switched[x][y] = Self::BLACK;
                } else if board[x][y] == Self::BLACK {
                    switched[x][y] = Self::WHITE;
                } else {
                    switched[x][y] = board[x][y];
                }
            }
        }
        switched
    }

    /// returns the current state with colors switched
    /// 
    /// this function is to predict move operations with always considering
    /// whtie as the color that has to move.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// # let abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// let switched_state = abalone.get_switched_state();
    /// ```
    pub fn get_switched_state(&self) -> Board {
        if self.black_tomove {
            return Self::switch_colors(self.board);
        }
        self.board
    }

    /// calculates possible moves for the given marble coordinates
    /// 
    /// this function will return a vector containing all possible follow-up
    /// position if the selected marbles will be moved with the corresponding
    /// `MarbleMove`. If no valid moves are possible the returned vector will be empty.
    /// Moves are not valid if
    /// * more than three marbles are selected
    /// * one or more of the selected marbles are not of the active color
    /// * not all marbles are in one line
    /// * one of the coordinates does not point to valid board position
    /// 
    /// # Arguments
    /// 
    /// * `start_coords` - coordinates of the selected marbles
    /// 
    /// # Returns
    /// 
    /// vector of tuples, each tuple contains
    /// * `marb_move` - the move the selected marbles will make to reach the position given
    /// * `next_state` - the corresponding state that will be reachwed after the move
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// # let abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// let coords = vec![Coord::new(3, 7), Coord::new(3, 8)];
    /// let possible_moves = abalone.calc_coord_moves(coords);
    /// ```
    pub fn calc_coord_moves(&self, start_coords: Vec<Coord>) -> HashMap<MarbleMove, Board> {
        let mut results: HashMap<MarbleMove, Board> = HashMap::with_capacity(6);
        match start_coords.len() {
            1 => self.single_coord_moves(start_coords, &mut results),
            2|3 => self.multi_coord_moves(start_coords, &mut results),
            _ => {}
        }
        // if black is to move, it is necessary to switch the colors back
        if self.black_tomove {
            for value in results.values_mut() {
                *value = Self::switch_colors(*value);
            }
        }
        return results;
    }

    fn single_coord_moves(&self, mut start_coords: Vec<Coord>, results: &mut HashMap<MarbleMove, Board>) {
        let start_coord = start_coords.pop().unwrap();
        // be sure that the given position can be valid at all
        if !start_coord.in_board_range() {
            return;
        }
        let switched = self.get_switched_state();
        // if the chosen position is not a white marble just return
        if switched[start_coord.x][start_coord.y] != Self::WHITE {
            return;
        }
        for marb_move in Self::MOVES.iter() {
            let target = start_coord + *marb_move;
            if switched[target.x][target.y] == Self::EMPTY {
                let mut new_board = switched;
                new_board[start_coord.x][start_coord.y] = Self::EMPTY;
                new_board[target.x][target.y] = Self::WHITE;
                results.insert(marb_move.clone(), new_board);
            }
        }
    }

    fn multi_coord_moves(&self, mut start_coords: Vec<Coord>, results: &mut HashMap<MarbleMove, Board>) {
        // sort marbles first
        start_coords.sort();
        let mut row_check = false;
        let first = start_coords[0];
        // if the first marble is not on a valid field just return
        if !first.in_board_range() {
            return;
        }
        let second = start_coords[1];
        let last = start_coords[start_coords.len()-1];
        let switched = self.get_switched_state();
        // if one of the marbles is not a white marble return
        for sc in start_coords.as_slice() {
            if switched[sc.x][sc.y] != Self::WHITE {
                return;
            }
        }

        for marb_move in Self::MOVES.iter() {
            let forward_pos = first + *marb_move;
            let backward_pos = first - *marb_move;
            if forward_pos == second {
                // first and second marble are fine at this point
                for index in 2..start_coords.len() {
                    if start_coords[index] != first.multi_move(marb_move, index) {
                        results.clear();
                        return;
                    }
                }
                // if function did not return, row_check worked
                row_check = true;
                // if second is in move direction, then 'last' is the front marble
                self.inline_move(switched, last, first, start_coords.len(), marb_move.clone(), results);
            } else if backward_pos == second {
                // if second is in opposite of move direction, then 'first' is the front marble
                self.inline_move(switched,first, last, start_coords.len(), marb_move.clone(), results);
            } else {
                // in any other case broadside moves are performed
                let mut pos_state = switched;
                let mut to_push = true;
                for marb_pos in start_coords.as_slice() {
                    let new_pos = *marb_pos + *marb_move;
                    if switched[new_pos.x][new_pos.y] == Self::EMPTY {
                        pos_state[marb_pos.x][marb_pos.y] = Self::EMPTY;
                        pos_state[new_pos.x][new_pos.y] = Self::WHITE;
                    } else {
                        to_push = false;
                        break;
                    }
                }
                if to_push {
                    results.insert(marb_move.clone(), pos_state);
                }
            }
        
        if !row_check {
            results.clear();
        }

        }
    }

    fn inline_move(&self, state: Board, front: Coord, back: Coord, num_marbs: usize, m_move: MarbleMove, results: &mut HashMap<MarbleMove, Board>) {
        let tar1 = front + m_move;
        if state[tar1.x][tar1.y] == Self::EMPTY {
            let mut pos_state = state;
            pos_state[tar1.x][tar1.y] = Self::WHITE;
            pos_state[back.x][back.y] = Self::EMPTY;
            results.insert(m_move, pos_state);
        } else if state[tar1.x][tar1.y] == Self::BLACK {
            let tar2 = tar1 + m_move;
            if state[tar2.x][tar2.y] == Self::EMPTY {
                let mut pos_state = state;
                pos_state[tar2.x][tar2.y] = Self::BLACK;
                pos_state[tar1.x][tar1.y] = Self::WHITE;
                pos_state[back.x][back.y] = Self::EMPTY;
                results.insert(m_move, pos_state);
            } else if state[tar2.x][tar2.y] == Self::OFF_BOARD {
                let mut pos_state = state;
                pos_state[tar1.x][tar1.y] = Self::WHITE;
                pos_state[back.x][back.y] = Self::EMPTY;
                results.insert(m_move, pos_state);
            } else if state[tar2.x][tar2.y] == Self::BLACK && num_marbs > 2 {
                let tar3 = tar2 + m_move;
                if state[tar3.x][tar3.y] == Self::EMPTY {
                    let mut pos_state = state;
                    pos_state[tar3.x][tar3.y] = Self::BLACK;
                    pos_state[tar1.x][tar1.y] = Self::WHITE;
                    pos_state[back.x][back.y] = Self::EMPTY;
                    results.insert(m_move, pos_state);
                } else if state[tar3.x][tar3.y] == Self::OFF_BOARD {
                    let mut pos_state = state;
                    pos_state[tar1.x][tar1.y] = Self::WHITE;
                    pos_state[back.x][back.y] = Self::EMPTY;
                    results.insert(m_move, pos_state);
                }
            }
        }
    }

    /// calculates the possible child states from the current board state
    ///
    /// All possible child states (excluding states which result from pushing
    /// the own marbles off the board) are calculated and internaly stored.
    /// The corresponding move IDs are returned
    ///
    /// # Returns
    /// * `pov_board` - board state from white's perspective
    /// * `next_moveids` - IDs for all allowed moves
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// # let mut abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// let (pov_state, move_ids) = abalone.calc_reasonalbe_moves();
    /// ```
    pub fn calc_reasonalbe_moves(&mut self) -> (Board, Vec<usize>) {
        let pov_state = self.get_rotated_state();
        self.next_positions.clear();
        let mut next_moveids: Vec<usize> = Vec::with_capacity(Self::MAX_SAVE);
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                if pov_state[x][y] != Self::WHITE {
                    continue;
                }
                let pos = Coord::new(x, y);
                for (m, marb_move) in Self::MOVES.iter().enumerate() {
                    let new_pos = pos + *marb_move;
                    if pov_state[new_pos.x][new_pos.y] == Self::EMPTY {
                        let neigh1_marb = pos - *marb_move;
                        if pov_state[neigh1_marb.x][neigh1_marb.y] == Self::WHITE {
                            next_moveids.push(self.move_straight_or_push_off(
                                pov_state,
                                neigh1_marb,
                                new_pos,
                                marb_move.dx,
                                marb_move.dy,
                            ));

                            let neigh2_marb = neigh1_marb - *marb_move;
                            if pov_state[neigh2_marb.x][neigh2_marb.y] == Self::WHITE {
                                next_moveids.push(self.move_straight_or_push_off(
                                    pov_state,
                                    neigh2_marb,
                                    new_pos,
                                    marb_move.dx,
                                    marb_move.dy,
                                ));
                            }
                        }
                    } else if pov_state[new_pos.x][new_pos.y] == Self::BLACK {
                        let neigh1_marb = pos - *marb_move;
                        let neigh2_marb = neigh1_marb - *marb_move;
                        if pov_state[neigh1_marb.x][neigh1_marb.y] == Self::WHITE {
                            let target = new_pos + *marb_move;
                            if pov_state[target.x][target.y] == Self::OFF_BOARD {
                                next_moveids.push(self.move_straight_or_push_off(
                                    pov_state,
                                    neigh1_marb,
                                    new_pos,
                                    marb_move.dx,
                                    marb_move.dy,
                                ));
                                if pov_state[neigh2_marb.x][neigh2_marb.y] == Self::WHITE {
                                    next_moveids.push(self.move_straight_or_push_off(
                                        pov_state,
                                        neigh2_marb,
                                        new_pos,
                                        marb_move.dx,
                                        marb_move.dy,
                                    ));
                                }
                            } else if pov_state[target.x][target.y] == Self::EMPTY {
                                next_moveids.push(self.move_push_empty(
                                    pov_state,
                                    neigh1_marb,
                                    new_pos,
                                    target,
                                    marb_move.dx,
                                    marb_move.dy,
                                ));
                                if pov_state[neigh2_marb.x][neigh2_marb.y] == Self::WHITE {
                                    next_moveids.push(self.move_push_empty(
                                        pov_state,
                                        neigh2_marb,
                                        new_pos,
                                        target,
                                        marb_move.dx,
                                        marb_move.dy,
                                    ));
                                }
                            } else if pov_state[target.x][target.y] == Self::BLACK
                                && pov_state[neigh2_marb.x][neigh2_marb.y] == Self::WHITE
                            {
                                let beyond = target + *marb_move;
                                if pov_state[beyond.x][beyond.y] == Self::OFF_BOARD {
                                    next_moveids.push(self.move_straight_or_push_off(
                                        pov_state,
                                        neigh2_marb,
                                        new_pos,
                                        marb_move.dx,
                                        marb_move.dy,
                                    ));
                                } else if pov_state[beyond.x][beyond.y] == Self::EMPTY {
                                    next_moveids.push(self.move_push_empty(
                                        pov_state,
                                        neigh2_marb,
                                        new_pos,
                                        beyond,
                                        marb_move.dx,
                                        marb_move.dy,
                                    ));
                                }
                            }
                        }
                    }
                    // broad side moves
                    for (s, side_move) in Self::ORTHO_MOVES[m].iter().enumerate() {
                        let mut new_board = pov_state;
                        let mut moved_pos = "".to_string();
                        for b in 0..Self::MARBLE_ROW {
                            let mar_pos = pos.multi_move(side_move, b);
                            if pov_state[mar_pos.x][mar_pos.y] == Self::WHITE {
                                let target = mar_pos + *marb_move;
                                if pov_state[target.x][target.y] == Self::EMPTY {
                                    new_board[mar_pos.x][mar_pos.y] = Self::EMPTY;
                                    new_board[target.x][target.y] = Self::WHITE;
                                } else {
                                    break;
                                }
                                // avoids pushing the same single marble move twice
                                if s > 0 && b == 0 {
                                    continue;
                                }
                                self.next_positions.push(new_board);
                                next_moveids.push(
                                    match marble_moves::MOVES_IDX.get(
                                        // the key consists of the already moved marbles combined with the current one
                                        format!(
                                            "{}{}{}{}{}",
                                            moved_pos,
                                            mar_pos.x,
                                            mar_pos.y,
                                            marb_move.dx,
                                            marb_move.dy
                                        )
                                        .as_str(),
                                    ) {
                                        Some(idx) => *idx,
                                        None => panic!("A non existent move ID was created"),
                                    },
                                );
                                // add current marble to the alreadyd moved marble
                                moved_pos = format!("{}{}{}", moved_pos, mar_pos.x, mar_pos.y);
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
        (pov_state, next_moveids)
    }

    fn move_straight_or_push_off(
        &mut self,
        state: Board,
        base: Coord,
        target: Coord,
        dx: i8,
        dy: i8,
    ) -> usize {
        self.next_positions.push(state);
        let last_added: &mut Board = self.next_positions.last_mut().unwrap();
        last_added[base.x][base.y] = Self::EMPTY;
        last_added[target.x][target.y] = Self::WHITE;
        match marble_moves::MOVES_IDX.get(format!("{}{}{}{}", base.x, base.y, dx, dy).as_str()) {
            Some(idx) => *idx,
            None => panic!("A non existent move ID was created"),
        }
    }

    fn move_push_empty(
        &mut self,
        state: Board,
        base: Coord,
        black: Coord,
        target: Coord,
        dx: i8,
        dy: i8,
    ) -> usize {
        self.next_positions.push(state);
        let last_added: &mut Board = self.next_positions.last_mut().unwrap();
        last_added[base.x][base.y] = Self::EMPTY;
        last_added[black.x][black.y] = Self::WHITE;
        last_added[target.x][target.y] = Self::BLACK;
        match marble_moves::MOVES_IDX.get(format!("{}{}{}{}", base.x, base.y, dx, dy).as_str()) {
            Some(idx) => *idx,
            None => panic!("A non existent move ID was created"),
        }
    }

    /// standard getter to obtain one of the child positions
    ///
    /// # Arguments
    ///
    /// * `index` - index of the desired position
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// # let mut abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// abalone.calc_reasonalbe_moves();
    /// let chosen_sate = abalone.get_next_position(0);
    /// ```
    ///
    /// # Panics
    ///
    /// will panic if the index is out of range
    pub fn get_next_position(&self, index: usize) -> Board {
        self.next_positions[index]
    }

    /// changes the current state to the given state
    ///
    /// It is possible to enter an invalid board state
    /// and the function will not check for that. This
    /// include generally impossible positions and positions
    /// that cannot be achieved by a move in the current
    /// position.
    ///
    /// # Arguments
    ///
    /// * `new_board` - upcoming position from white's point of view
    ///
    /// # Examples
    /// ```
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// # let mut abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// let new_board = [
    ///     [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    ///     [3, 3, 3, 3, 3, 2, 2, 0, 1, 1, 3],
    ///     [3, 3, 3, 3, 2, 2, 2, 1, 1, 1, 3],
    ///     [3, 3, 3, 0, 2, 2, 0, 1, 1, 0, 3],
    ///     [3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 3],
    ///     [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3],
    ///     [3, 0, 1, 0, 0, 0, 0, 0, 0, 3, 3],
    ///     [3, 0, 1, 1, 0, 2, 2, 0, 3, 3, 3],
    ///     [3, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3],
    ///     [3, 1, 0, 0, 2, 2, 3, 3, 3, 3, 3],
    ///     [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    /// ];
    /// abalone.update_state(new_board);
    /// ```
    pub fn update_state(&mut self, mut new_board: Board) {
        if self.black_tomove {
            new_board = Self::rotate_board(new_board);
        }

        self.board = new_board;
        self.black_tomove = !self.black_tomove;

        let mut noloss: bool = true;
        let white_newloss = Self::MARBLES_MAX - Self::count_marbles(new_board, Self::WHITE);
        let black_newloss = Self::MARBLES_MAX - Self::count_marbles(new_board, Self::BLACK);

        if white_newloss > self.white_loss {
            self.white_loss = white_newloss;
            noloss = false;
        } else if black_newloss > self.black_loss {
            self.black_loss = black_newloss;
            noloss = false;
        }

        if noloss {
            self.noloss_moves += 1;
            if !self.black_tomove && self.noloss_moves > 1 {
                self.noloss_turns += 1;
            }
        } else {
            self.noloss_moves = 0;
            self.noloss_turns = 0;
        }

        if self.save_history {
            self.state_history.push(new_board);
        }

        self.state_memory
            .entry(new_board)
            .and_modify(|count| *count += 1)
            .or_insert(1);
        self.check_game_ended();
    }

    /// updates the board state according to the given index
    ///
    /// In order for this function to work, it is necessary to call
    /// `calc_reasonalbe_moves` in order to generate the follow-up
    /// positions for the current state.
    ///
    /// # Arguments
    ///
    /// * `index` - index of the desired child position
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// # let mut abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// let (_pov_state, _move_ids) = abalone.calc_reasonalbe_moves();
    /// abalone.update_by_id(0);
    /// ```
    /// # Panics
    ///
    /// will panic if the given `index` is out of bounds
    pub fn update_by_id(&mut self, index: usize) {
        self.update_state(self.get_next_position(index));
    }

    /// sets the game result to the given value and ends the game
    /// 
    /// # Arguments
    /// 
    /// * `result` - game result, can be `-1` => black wins, `0` => draw, or `1` => white wins
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use rustai_abalone::game::{AbaloneGame, BELGIAN_DAISY};
    /// # let mut abalone = AbaloneGame::new(BELGIAN_DAISY);
    /// abalone.end_with_result(1);
    /// ```
    pub fn end_with_result(&mut self, result: i8) {
        if result > 1 || result < -1 {
            return;
        }
        self.result = result;
        self.game_ended = true;
    }
}
