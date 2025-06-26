use crate::game::{AbaloneGame, Board};
use rand::distributions::WeightedIndex;
use rand::prelude::{thread_rng, Distribution};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::thread::JoinHandle;
use std::{thread, time};
use tensorflow::{
    Graph, Operation, SavedModelBundle, Session, SessionOptions, SessionRunArgs, Tensor,
};

use super::game;
use super::util;

/// agent that can play Abalone
pub struct MagisterLudi {
    /// abalone game instance that the agent uses for internal representation
    abalone: game::AbaloneGame,
    /// stored path to the tensorflow model
    model_path: String,
    /// sender and receiver for thread manipulation
    stop_sender: Sender<bool>,
    stop_receiver: Receiver<bool>,
    /// tensorflow session for the main thread
    main_session: Session,
    /// tensorflow input operation for the main thread
    main_inp: Operation,
    /// tensorflow output operation for the action distribution within the main thread
    main_distr_out: Operation,
    /// number of leafs to be reached for the MCTS
    mcts_num: usize,
    /// number of threads used for the MCTS
    mcts_parallel: usize,
    /// denotes whether the current MCTS is finished or not
    mcts_finished: Arc<Mutex<usize>>,
    /// minimum number a follow-up state must be selected from the root state
    mcts_minimum: usize,
    /// denotes how many moves will be played until the expansion will be evaluated.
    /// If 0 the game will be played until the end.
    mcts_depth: usize,
    /// join handles for the MCTS threads
    mcts_handles: Vec<JoinHandle<()>>,
    /// map for storing the leaf results according to the selected position from the root state
    mcts_results: Arc<Mutex<HashMap<game::Board, f32>>>,
    /// map for storing the
    mcts_counts: HashMap<game::Board, f32>,
    /// map for storing
    mcts_board_ids: HashMap<usize, usize>,
    /// stores the distribution for a vistied state, as calcuating distributions is expansive
    saved_distr: Arc<Mutex<HashMap<game::Board, WeightedIndex<f32>>>>,
    /// vector that stores the selected position which will be simulated
    game_queue: Arc<Mutex<Vec<(game::AbaloneGame, game::Board)>>>,
}

impl MagisterLudi {
    /// creates a new agent instance and starts the necessary threads.
    /// Checks whether the required model is present in the given `model_path`.
    /// If it is not present it will be automatically downloaded.
    ///
    /// # Arguments
    ///
    /// * `board` - 11 x 11 array with the initial board position
    /// * `model_path` - optional path to the stored tensorflow model, if None the library folder will be selected
    /// * `mcts_num` - number of leafs for every MCTS
    /// * `mcts_parallel` - number of threads for the MCTS
    /// * `mcts_minimum` - denotes how often a child state must at least be selected
    /// * `mcts_depth` - number of moves that will be simulated starting for the root state,
    ///     if 0 simulations run until the games end
    ///
    /// # Examples
    /// 
    /// ```rust
    /// use rustai_abalone::player::MagisterLudi;
    /// let mut magister_ludi = MagisterLudi::new(rustai_abalone::game::BELGIAN_DAISY, None, 10, 5, 1, 5);
    /// # magister_ludi.stop_execution();
    /// ```
    /// 
    /// # Panics
    /// 
    /// will panic if the provided model path does not exist or if the model
    /// is not present and downloading it fails
    pub fn new(
        board: game::Board,
        model_path: Option<&str>,
        mcts_num: usize,
        mcts_parallel: usize,
        mcts_minimum: usize,
        mcts_depth: usize,
    ) -> Self {
        // if no path is given the library path will be used
        let model_path = match model_path {
            Some(val) => val,
            _ => &std::env::var("CARGO_MANIFEST_DIR").unwrap()
        };
        // panics, if the given path does not exist
        // searches the path for required files and will download them if not present
        let final_path = match util::check_model_present(model_path){
            Some(val) => val,
            _ => {
                // if the model is not present, download it
                util::download_model(model_path);
                // check whether the download worked, otherwise panic
                match util::check_model_present(model_path) {
                    Some(wal) => wal,
                    _ => panic!("Downloading model did not work!")
                }
            }
        };
        let (tx, rx) = unbounded();
        let (session, _, inp, distr_out, _) = Self::create_session(&final_path);
        let mut mag_ludi = Self {
            abalone: game::AbaloneGame::new(board),
            model_path: final_path.to_string(),
            stop_sender: tx,
            stop_receiver: rx,
            main_session: session,
            main_inp: inp,
            main_distr_out: distr_out,
            mcts_num,
            mcts_parallel,
            mcts_finished: Arc::new(Mutex::new(0)),
            mcts_minimum,
            mcts_depth,
            mcts_handles: Vec::with_capacity(mcts_parallel),
            mcts_results: Arc::new(Mutex::new(HashMap::new())),
            mcts_counts: HashMap::new(),
            mcts_board_ids: HashMap::new(),
            saved_distr: Arc::new(Mutex::new(HashMap::with_capacity(mcts_num * 150 * 150))),
            game_queue: Arc::new(Mutex::new(Vec::with_capacity(mcts_num))),
        };
        mag_ludi.start_threads(&final_path);
        mag_ludi
    }

    /// starts a new game for the agent
    /// 
    /// will start a fresh Abalone game for the given starting position and respawns
    /// the threads for execution of the MCTS
    /// 
    /// # Arguments
    /// 
    /// * `board` - starting position for the new game
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # let mut magister_ludi = rustai_abalone::player::MagisterLudi::new(rustai_abalone::game::BELGIAN_DAISY, None, 10, 5, 1, 5);
    /// magister_ludi.start_new_game(rustai_abalone::game::BELGIAN_DAISY);
    /// # magister_ludi.stop_execution();
    /// ```
    pub fn start_new_game(&mut self, board: game::Board) {
        if !self.check_threads_all_active() {
            // be sure that all threads stopped
            self.stop_execution();
            // restart threads
            let load_path = self.model_path.clone();
            self.start_threads(&load_path);
        }
        self.abalone = game::AbaloneGame::new(board);
    }

    /// lets the agent know that a move was made by an external source and change its game representation accordingly
    ///
    /// # Argmuents
    ///
    /// * `new_state` - next state of currently played game. The agent will not check whether this position is valid
    /// * `receive_pov` - if true, the next state will be considered to be from white's point of view
    ///
    /// # Examples
    ///
    /// ```rust
    /// # let mut magister_ludi = rustai_abalone::player::MagisterLudi::new(rustai_abalone::game::BELGIAN_DAISY, None, 10, 5, 1, 5);
    /// let next_state = [
    ///     [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    ///     [3, 3, 3, 3, 3, 1, 1, 0, 2, 2, 3],
    ///     [3, 3, 3, 3, 1, 1, 1, 2, 2, 2, 3],
    ///     [3, 3, 3, 0, 1, 1, 0, 2, 2, 0, 3],
    ///     [3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 3],
    ///     [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3],
    ///     [3, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3],
    ///     [3, 0, 2, 2, 0, 1, 1, 0, 3, 3, 3],
    ///     [3, 2, 2, 2, 1, 1, 1, 3, 3, 3, 3],
    ///     [3, 2, 2, 0, 1, 1, 3, 3, 3, 3, 3],
    ///     [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    /// ];
    /// magister_ludi.external_move(next_state, true);
    /// # magister_ludi.stop_execution();
    /// ```
    pub fn external_move(&mut self, mut new_state: Board, receive_pov: bool) {
        // do nothing if game has ended
        if self.abalone.get_game_ended() {
            return;
        }
        // consider rotation
        if !receive_pov && self.abalone.get_black_tomove() {
            new_state = AbaloneGame::rotate_board(new_state)
        }
        self.abalone.update_state(new_state);
        self.check_game_ended();
    }

    /// lets the agent know that a move was made by an external source and change its game representation accordingly
    /// 
    /// # Arguments
    /// 
    /// * `return_pov` - if true, chosen state will be returned from white's point of view
    ///
    /// # Returns
    ///
    /// * `chosen_state` - the state the agent wants to reach with its move
    ///
    /// # Examples
    ///
    /// ```rust
    /// # let mut magister_ludi = rustai_abalone::player::MagisterLudi::new(rustai_abalone::game::BELGIAN_DAISY, None, 10, 5, 1, 5);
    /// let next_state = magister_ludi.own_move(true);
    /// assert!(rustai_abalone::game::AbaloneGame::validate_board(next_state));
    /// # magister_ludi.stop_execution();
    /// ```
    /// 
    /// # Panics
    /// 
    /// will panic if one or more of the MCTS daemon threads are not active anymore
    pub fn own_move(&mut self, return_pov: bool) -> Board {
        if !self.check_threads_all_active() {
            panic!("Cannot execute move as there are issues with the activity of MCTS threads")
        }
        self.choose_possible_moves();
        self.push_to_queue();
        let chosen_state = self.choose_next_move();
        self.check_game_ended();
        // chosen_state is from white's point of view
        if return_pov {
            return chosen_state;
        }
        // otherwise return state as it is
        self.abalone.get_state()
    }

    // selects the child state which should be simulated from the root state
    fn choose_possible_moves(&mut self) {
        let saved_distr = self.saved_distr.clone();
        let mut rng = thread_rng();
        self.mcts_board_ids.clear();
        let (state, move_ids) = self.abalone.calc_reasonalbe_moves();
        // initialize 0 counts for all possible moves
        let mut board_ids: Vec<usize> = (0..move_ids.len()).collect();
        for idx in &board_ids {
            self.mcts_board_ids.insert(*idx, 0);
        }
        let mut distr_map = saved_distr.lock().unwrap();
        // create distribution
        let distr = match distr_map.get(&state) {
            Some(distr) => distr,
            None => {
                let distr = Self::calc_distribution(
                    &self.main_session,
                    &self.main_inp,
                    &self.main_distr_out,
                    &Self::convert_board_to_tensor(state),
                    &move_ids,
                );
                distr_map.insert(state, distr);
                distr_map.get(&state).unwrap()
            }
        };
        // use distribution to draw next moves
        for _ in 0..self.mcts_num {
            let count = self
                .mcts_board_ids
                .get_mut(&distr.sample(&mut rng))
                .unwrap();
            *count += 1;
        }

        // redistribute chosen next state ids to match minimum count
        board_ids.shuffle(&mut rng);
        for idx in &board_ids {
            let mut count = *self.mcts_board_ids.get_mut(idx).unwrap();
            if count < self.mcts_minimum {
                self.mcts_board_ids.insert(*idx, 0);
                while count > 0 {
                    let other_idx = distr.sample(&mut rng);
                    let other_count = self.mcts_board_ids.get_mut(&other_idx).unwrap();
                    if *other_count >= self.mcts_minimum - 1 {
                        *other_count += 1;
                        count -= 1;
                    }
                }
            }
        }
    }

    // pushes the chosen moves to the queue for simulation
    fn push_to_queue(&mut self) {
        let game_queue = self.game_queue.clone();
        self.mcts_results.lock().unwrap().clear();
        self.mcts_counts.clear();
        let mut queue = game_queue.lock().unwrap();
        // first push every position once to make greater use of saved distributions
        for (idx, count) in self.mcts_board_ids.iter() {
            if *count > 0 {
                let board = self.abalone.get_next_position(*idx);
                queue.push((self.abalone.mcts_copy(), board));
            }
        }
        for (idx, count) in self.mcts_board_ids.iter() {
            if *count > 0 {
                let board = self.abalone.get_next_position(*idx);
                for _ in 0..(count - 1) {
                    queue.push((self.abalone.mcts_copy(), board));
                }
                self.mcts_counts.insert(board, *count as f32);
            }
        }
    }

    // chooses the next move depending on the outcome of the MCTS
    fn choose_next_move(&mut self) -> Board {
        // await queue results
        let mcts_finished = self.mcts_finished.clone();
        *mcts_finished.lock().unwrap() = 0;

        let sleep_time = time::Duration::from_millis(100);
        while *mcts_finished.lock().unwrap() < self.mcts_num {
            thread::sleep(sleep_time);
        }

        let mcts_results = self.mcts_results.clone();
        let mut best_state = game::BELGIAN_DAISY;
        let mut best_result: f32 = -(self.mcts_num as f32); // worst result possible
        for (state, result_value) in mcts_results.lock().unwrap().iter_mut() {
            // the result value is averaged
            *result_value /= self.mcts_counts.get(state).unwrap();
            // this variant is for sure not the best, but it is also not slowing down the overall process
            if *result_value > best_result {
                best_result = *result_value;
                best_state = *state;
            }
        }
        self.abalone.update_state(best_state);
        best_state
    }

    // starts the threads for the MCTS when the class is initialized
    fn start_threads(&mut self, model_path: &str) {
        for i in 0..self.mcts_parallel {
            let t_receiver = self.stop_receiver.clone();
            let game_queue = self.game_queue.clone();
            let mcts_results = self.mcts_results.clone();
            let mcts_finished = self.mcts_finished.clone();
            let saved_distr = self.saved_distr.clone();
            let mcts_depth = self.mcts_depth;
            let main_black_tomove = self.abalone.get_black_tomove();
            let thread_path = model_path.to_string();

            let handle = thread::spawn(move || {
                let sleep_time = time::Duration::from_millis(500);
                // create model for each thread
                let (session, _graph, inp, distr_out, rating_out) = Self::create_session(&thread_path);
                let mut rng = thread_rng();

                loop {
                    if let Ok(signal) = t_receiver.try_recv() {
                        if signal {
                            println!("Thread {i} terminating");
                            break;
                        }
                    }

                    let leaf_entry = game_queue.lock().unwrap().pop();
                    match leaf_entry {
                        Some(entry) => {
                            let mut moves_performed: usize = 0;
                            let (mut aba, next_board) = entry;
                            aba.update_state(next_board);
                            while !aba.get_game_ended() {
                                let (state, move_ids) = aba.calc_reasonalbe_moves();
                                let index_opt = saved_distr
                                    .lock()
                                    .unwrap()
                                    .get(&state)
                                    .map(|distr| distr.sample(&mut rng));
                                let brd_index = match index_opt {
                                    Some(idx) => idx,
                                    None => {
                                        let distr = Self::calc_distribution(
                                            &session,
                                            &inp,
                                            &distr_out,
                                            &Self::convert_board_to_tensor(state),
                                            &move_ids,
                                        );
                                        let idx = distr.sample(&mut rng);
                                        saved_distr.lock().unwrap().insert(state, distr);
                                        idx
                                    }
                                };
                                let next_state = aba.get_next_position(brd_index);
                                aba.update_state(next_state);
                                moves_performed += 1;
                                // will never be true for self.mcts_depth == 0
                                if moves_performed == mcts_depth || aba.get_game_ended() {
                                    break;
                                }
                            }
                            let black_factor_main: f32 = if main_black_tomove { -1.0 } else { 1.0 };
                            let mut leaf_result: f32 = if aba.get_game_ended() {
                                aba.get_game_result().into()
                            } else {
                                let black_factor_leaf: f32 =
                                    if aba.get_black_tomove() { -1.0 } else { 1.0 };
                                let state = aba.get_rotated_state();
                                Self::calc_rating(
                                    &session,
                                    &inp,
                                    &rating_out,
                                    &Self::convert_board_to_tensor(state),
                                ) * black_factor_leaf
                            };
                            leaf_result *= black_factor_main;
                            mcts_results.lock().unwrap().insert(next_board, leaf_result);
                            *mcts_finished.lock().unwrap() += 1;
                        }
                        None => {
                            thread::sleep(sleep_time);
                        }
                    };
                }
            });
            self.mcts_handles.push(handle);
        }
    }

    /// checks whether all daemon threads for the MCTS are still running
    /// 
    /// # Returns
    /// 
    /// * `all_active` true if all daemon MCTS threads are still running, else false
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # let mut magister_ludi = rustai_abalone::player::MagisterLudi::new(rustai_abalone::game::BELGIAN_DAISY, None, 10, 5, 1, 5);
    /// let all_active = magister_ludi.check_threads_all_active();
    /// # magister_ludi.stop_execution();
    /// ```
    pub fn check_threads_all_active(& self) -> bool {
        let mut num_active: usize = 0;
        for handle in self.mcts_handles.as_slice() {
            if !handle.is_finished() {
                num_active += 1;
            }
        }
        if num_active == self.mcts_parallel {
            return true;
        }
        false
    }

    // creates a tensorflow session and input and output operations for the model
    fn create_session(model_path: &str) -> (Session, Graph, Operation, Operation, Operation) {
        let signature_input_parameter_name = "input_8"; // adjust
        let signature_output_distr_name = "pol_prediction"; // adjust
        let signature_output_rating_name = "val_prediction"; // adjust

        // Initialize save_dir, input tensor, and an empty graph
        // let save_dir = "C:\\Users\\hlocke\\Documents\\Repos\\UdemyRust\\MyTasks\\tf_ludi\\src\\magister_zero_unwrap_save";

        let mut graph = Graph::new();

        // Load saved model bundle (session state + meta_graph data)
        let bundle =
            SavedModelBundle::load(&SessionOptions::new(), ["serve"], &mut graph, model_path)
                .expect("Can't load saved model");

        // Get signature metadata from the model bundle
        let signature = bundle
            .meta_graph_def()
            .get_signature("serving_default")
            .unwrap();

        // Get input/output info
        let input_info = signature.get_input(signature_input_parameter_name).unwrap();
        let output_distr_info = signature.get_output(signature_output_distr_name).unwrap();
        let output_rating_info = signature.get_output(signature_output_rating_name).unwrap();

        // Get input/output ops from graph
        let input_op = graph
            .operation_by_name_required(&input_info.name().name)
            .unwrap();
        let output_distr_op = graph
            .operation_by_name_required(&output_distr_info.name().name)
            .unwrap();
        let output_rating_op = graph
            .operation_by_name_required(&output_rating_info.name().name)
            .unwrap();

        // Get the session from the loaded model bundle
        let session = bundle.session;
        (session, graph, input_op, output_distr_op, output_rating_op)
    }

    // prepares the current board for tensorflow input
    fn convert_board_to_tensor(board: game::Board) -> Tensor<f32> {
        let mut tensor: Tensor<f32> =
            Tensor::new(&[1, game::BOARD_SIZE as u64, game::BOARD_SIZE as u64, 4]);
        for x in 0..game::BOARD_SIZE {
            for y in 0..game::BOARD_SIZE {
                tensor.set(&[0, x as u64, y as u64, board[x][y] as u64], 1.0);
            }
        }
        tensor
    }

    // calcuates the action/move distribution for a given position
    fn calc_distribution(
        session: &Session,
        input: &Operation,
        distr_output: &Operation,
        tensor: &Tensor<f32>,
        move_ids: &[usize],
    ) -> WeightedIndex<f32> {
        let mut args = SessionRunArgs::new();
        args.add_feed(input, 0, tensor); // Add any inputs

        let out = args.request_fetch(distr_output, 0); // Request outputs

        // Run model
        session
            .run(&mut args) // Pass to session to run
            .expect("Error occurred during calculations");

        // Fetch outputs after graph execution
        let move_logits: Tensor<f32> = args.fetch(out).unwrap();

        // use output to calculate distribution by softmax
        let move_exp: Vec<f32> = move_ids.iter().map(|idx| move_logits[*idx].exp()).collect();
        let move_exp_sum: f32 = move_exp.iter().sum();
        let weights: Vec<f32> = move_exp.iter().map(|val| val / move_exp_sum).collect();
        WeightedIndex::new(weights).unwrap()
    }

    // calculates the evaluation for given position
    fn calc_rating(
        session: &Session,
        input: &Operation,
        rating_output: &Operation,
        tensor: &Tensor<f32>,
    ) -> f32 {
        let mut args = SessionRunArgs::new();
        args.add_feed(input, 0, tensor); // Add any inputs

        let out = args.request_fetch(rating_output, 1); // Request output

        // Run model
        session
            .run(&mut args) // Pass to session to run
            .expect("Error occurred during calculations");

        // Fetch outputs after graph execution
        args.fetch(out).unwrap()[0]
    }

    // checks whether the inner game representation of the agent ended by win, loss or draw
    fn check_game_ended(&mut self) {
        let game_ended = self.abalone.get_game_ended();
        if game_ended {
            for _ in 0..self.mcts_parallel {
                self.stop_sender.send(true).unwrap();
            }
            while let Some(handle) = self.mcts_handles.pop() {
                handle.join().unwrap();
            }
        }
    }

    /// sets the inner game representation result to a draw and stops the daemon threads
    /// 
    /// # Examples
    ///
    /// ```rust
    /// # let mut magister_ludi = rustai_abalone::player::MagisterLudi::new(rustai_abalone::game::BELGIAN_DAISY, None, 10, 5, 1, 5);
    /// magister_ludi.stop_execution();
    /// ```
    pub fn stop_execution(&mut self) {
        self.abalone.end_with_result(0);
        self.check_game_ended();
    }
}
