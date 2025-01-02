use crate::game::Board;
use rand::distributions::WeightedIndex;
use rand::prelude::{thread_rng, Distribution};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{thread, time};
use tensorflow::{
    Graph, Operation, SavedModelBundle, Session, SessionOptions, SessionRunArgs, Tensor,
};

use super::game;

struct MagisterLudi {
    abalone: game::AbaloneGame,
    main_session: Session,
    main_graph: Graph,
    main_inp: Operation,
    main_distr_out: Operation,
    main_rating_out: Operation,
    mcts_num: usize,
    mcts_minimum: usize,
    mcts_depth: usize,
    mcts_sessions: Vec<(Session, Graph, Operation, Operation, Operation)>,
    mcts_handles: Vec<JoinHandle<()>>,
    mcts_results: HashMap<game::Board, f32>,
    mcts_counts: HashMap<game::Board, f32>,
    mcts_board_ids: HashMap<usize, usize>,
    saved_distr: HashMap<game::Board, WeightedIndex<f32>>,
    game_queue: Arc<Mutex<Vec<(game::AbaloneGame, game::Board)>>>, //game_queue: Vec<(game::AbaloneGame, game::Board)>,
}

impl MagisterLudi {
    pub fn new(
        board: game::Board,
        mcts_num: usize,
        mcts_minimum: usize,
        mcts_depth: usize,
    ) -> Self {
        let (session, graph, inp, distr_out, rating_out) = Self::create_session();
        let mut mag_ludi = Self {
            abalone: game::AbaloneGame::new(board),
            main_session: session,
            main_graph: graph,
            main_inp: inp,
            main_distr_out: distr_out,
            main_rating_out: rating_out,
            mcts_num,
            mcts_minimum,
            mcts_depth,
            mcts_sessions: Vec::with_capacity(mcts_num),
            mcts_handles: Vec::with_capacity(mcts_num),
            mcts_results: HashMap::new(),
            mcts_counts: HashMap::new(),
            mcts_board_ids: HashMap::new(),
            saved_distr: HashMap::with_capacity(mcts_num * 150 * 150),
            game_queue: Arc::new(Mutex::new(Vec::with_capacity(mcts_num))),
            //game_queue: Vec::with_capacity(mcts_num),
        };
        mag_ludi.start_threads();
        mag_ludi
    }
    pub fn external_move(&mut self, new_state: Board) {
        // consider rotation
        self.abalone.update_state(new_state);
    }
    pub fn own_move(&mut self) {
        let mut rng = thread_rng();
        self.mcts_results.clear();
        self.mcts_counts.clear();
        self.mcts_board_ids.clear();
        let state = self.abalone.get_rotated_state();
        let move_ids = self.abalone.calc_reasonalbe_moves(state);
        // initialize 0 counts for all possible moves
        let mut board_ids: Vec<usize> = (0..move_ids.len()).collect();
        for idx in &board_ids {
            self.mcts_board_ids.insert(*idx, 0);
        }
        let distr = match self.saved_distr.get(&state) {
            Some(distr) => distr,
            None => {
                let distr = Self::calc_distribution(
                    &self.main_session,
                    &self.main_inp,
                    &self.main_distr_out,
                    &Self::convert_board_to_tensor(state),
                    &move_ids,
                );
                self.saved_distr.insert(state, distr);
                self.saved_distr.get(&state).unwrap()
            }
        };
        // choose moves
        for _ in 0..self.mcts_num {
            let idx = distr.sample(&mut rng);
            let count = self.mcts_board_ids.get_mut(&idx).unwrap();
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

        // finally add the chosen games to the queue
        for (idx, count) in self.mcts_board_ids.iter() {
            let board = self.abalone.get_next_position(*idx);
        }
    }

    fn start_threads(&mut self) {
        for thread_id in 0..self.mcts_num {
            self.mcts_sessions.push(Self::create_session());
            let handle = thread::spawn(|| {
                self.mcts(thread_id);
            });
            self.mcts_handles.push(handle);
        }
    }

    fn mcts(&mut self, thread_id: usize) {
        let sleep_time = time::Duration::from_millis(100);
        // create model for each thread
        let (session, _graph, inp, distr_out, rating_out) = &self.mcts_sessions[thread_id];
        let mut rng = thread_rng();

        /*
        while loop -> main game not ended
        - receive current game from some sort of queue
        - play game to the end
          * predict or recieve move distribution
          * choose random move from distribution
          * perform move
        - save outcome
         */
        while !self.abalone.get_game_ended() {
            let leaf_entry = self.game_queue.lock().unwrap().pop();
            match leaf_entry {
                Some(entry) => {
                    let mut moves_performed: usize = 0;
                    let (mut aba, next_board) = entry;
                    aba.update_state(next_board);
                    while aba.get_game_ended() {
                        let state = aba.get_rotated_state();
                        let move_ids = aba.calc_reasonalbe_moves(state);
                        let distr = match self.saved_distr.get(&state) {
                            Some(distr) => distr,
                            None => {
                                let distr = Self::calc_distribution(
                                    session,
                                    inp,
                                    distr_out,
                                    &Self::convert_board_to_tensor(state),
                                    &move_ids,
                                );
                                self.saved_distr.insert(state, distr);
                                self.saved_distr.get(&state).unwrap()
                            }
                        };
                        let next_state = aba.get_next_position(distr.sample(&mut rng));
                        aba.update_state(next_state);
                        moves_performed += 1;
                        // will never be true for self.mcts_depth = 0
                        if moves_performed == self.mcts_depth {
                            break;
                        }
                    }
                    let mut leaf_result: f32 = 0.0;
                    let black_factor_main: f32 = if self.abalone.get_black_tomove() {
                        -1.0
                    } else {
                        1.0
                    };
                    if aba.get_game_ended() {
                        leaf_result = aba.get_game_result().into();
                        if self.abalone.get_black_tomove() {
                            leaf_result *= -1.0;
                        }
                    } else {
                        let state = aba.get_rotated_state();
                        leaf_result = Self::calc_rating(
                            session,
                            inp,
                            rating_out,
                            &Self::convert_board_to_tensor(state),
                        );
                        let black_factor_leaf: f32 =
                            if aba.get_black_tomove() { -1.0 } else { 1.0 };
                        leaf_result += black_factor_leaf;
                    }
                    leaf_result *= black_factor_main;
                }
                None => {
                    thread::sleep(sleep_time);
                }
            };
        }

        // move distribution
    }

    fn create_session() -> (Session, Graph, Operation, Operation, Operation) {
        let signature_input_parameter_name = "test_in_input"; // adjust
        let signature_output_distr_name = "test_distr_out"; // adjust
        let signature_output_rating_name = "test_rating_out"; // adjust

        // Initialize save_dir, input tensor, and an empty graph
        let save_dir = "C:\\Users\\hlocke\\"; // adjust

        let mut graph = Graph::new();

        // Load saved model bundle (session state + meta_graph data)
        let bundle =
            SavedModelBundle::load(&SessionOptions::new(), ["serve"], &mut graph, save_dir)
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

    fn calc_distribution(
        session: &Session,
        input: &Operation,
        distr_output: &Operation,
        tensor: &Tensor<f32>,
        move_ids: &Vec<usize>,
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
}
