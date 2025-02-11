//! rustai abalone library.
//!
//! Conatins an Abalone game implementation and an agent that can play the game.
//! The game implementation allows to calculate all feasible follow-up positions.
//! The board state is represented by a 11 x 11 array. 0 denotes empty board fields,
//! 1 stands for white marbles, 2 for black marbles, and 3 denotes "off-board" positions.
//! In order to achieve the hexagonal shape and savely calculate new states, it is necessary
//! that the "edges" of the array are "off-board"
//!
//! The agent is based on the concept of Alpha-Zero. If the agents is required to make a move
//! it will calculate a distribution for all possible moves according to the predicted chance
//! to win the game with that move and randomly draws a certain amount of moves from this distribution.
//! The drawn child states from this root state will be simulated by the means of an MCTS and
//! the results of these simulations will be averaged to predict the move with the highest
//! winning chance.

pub mod game;
pub mod marble_moves;
pub mod player;
pub mod util;

#[cfg(test)]
mod tests {
    use util::{download_model, check_model_present};
    use std::path::Path;
    use game::AbaloneGame;
    use player::MagisterLudi;

    use super::*;

    #[test]
    fn test_download() {
        let dl_folder = Path::new(".").join("test_download");
        let path_to_model = download_model(dl_folder.to_str().unwrap());
        let check = check_model_present(&path_to_model).is_some();
        assert!(check);
    }

    #[test]
    fn test_abalone_game() {
        // change board to something with all possibilities
        let board = [
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
        // create
        let mut abalone = AbaloneGame::new(board);
        // should panic if something is not right, e.g. the board has no frame of "offboard" fields
        // receive the calculated next moves
        let (_state, move_ids) = abalone.calc_reasonalbe_moves();

        assert!(!move_ids.is_empty());

        // get next position and assert, that there are as many positions as move IDs
        let next_pos = abalone.get_next_position(move_ids.len() - 1);

        // check whether update works
        abalone.update_state(next_pos);

        // create copy for mcts
        let leaf_copy = abalone.mcts_copy();

        // get state for mcts initializing
        let state = abalone.get_rotated_state();
        let leaf_state = leaf_copy.get_rotated_state();

        // check whether copy gives the same
        assert_eq!(state, leaf_state);

        let game_end = abalone.get_game_ended();
        assert!(!game_end);

        abalone.calc_reasonalbe_moves();
        let next_pos = abalone.get_next_position(0);
        abalone.update_state(next_pos);
    }

    #[test]
    fn test_magister_ludi() {
        let mut magi_ludi = MagisterLudi::new(game::BELGIAN_DAISY, None, 10, 5, 1, 5);
        println!("initialized succesfully");
        let chosen_move = magi_ludi.own_move(true);
        assert!(AbaloneGame::validate_board(chosen_move));
        println!("Finished move");
        magi_ludi.start_new_game(game::BELGIAN_DAISY);
        println!("Started new game");
        magi_ludi.stop_execution();
        println!("Stopped execution");
    }
}
