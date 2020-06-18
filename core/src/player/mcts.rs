use crate::board::*;
use crate::player::*;
use std::cell::RefCell;
use rand::seq::SliceRandom;
use std::rc::Rc;

struct Node {
    wins: f32,
    visited: f32,
    action: Vec<MOVE>,
    mv: MOVE,
    pub untried_moves: Vec<usize>,
    parent: Option<Rc<RefCell<Node>>>,
    children: Vec<Option<Rc<RefCell<Node>>>>,
}

impl Node {
    fn new(state: Rc<RefCell<Board>>, parent: Option<Rc<RefCell<Node>>>, mv: MOVE) -> Self {
        let action = state.borrow().generate_all_steps();
        let mut node = Self {
            wins: 0.0,
            visited: 0.0,
            mv,
            untried_moves: (0..action.len()).collect(),
            children: vec![None; action.len()],
            action,
            parent,
        };
        node.untried_moves.shuffle(&mut rand::thread_rng());
        node
    }

    fn uct_select_child(&self) -> usize {
        let mut action_idx = 0;
        let mut max_uct_value = 0.0;
        for (idx, c) in self.children.iter().enumerate() {
            let c = c.as_ref().unwrap().borrow();

            let uct_value = c.wins / c.visited + ((2.0 * self.visited.log2()) / c.visited).sqrt();
            if uct_value > max_uct_value {
                max_uct_value = uct_value;
                action_idx = idx;
            }

        }

        action_idx
    }

    fn update(&mut self, result: f32) {
        self.visited += 1.0;
        self.wins += result;
    }
}



pub struct MCTSPlayer {
    board: Rc<RefCell<Board>>,
}

impl MCTSPlayer {
    pub fn new(board: Rc<RefCell<Board>>) -> Self {
        Self {
            board: board.clone()
        }
    }

    fn mcts_run(&mut self, itermax: usize) -> MOVE {
        let state = self.board.clone();
        let root = Rc::new(RefCell::new(Node::new(
            state.clone(), None,
            0
        )));

        for _iter in 0..itermax {
            let mut node = Some(root.clone());
            let mut steps = 0;
            // println!("iter: {} action.len={}", _iter, root.borrow().action.len());

            // select
            {
                while node.clone().unwrap().borrow().untried_moves.is_empty() &&
                    ! node.clone().unwrap().borrow().children.is_empty()
                {
                    let best_move_idx = node.clone().unwrap().borrow().uct_select_child();
                    state.borrow_mut().move_chess(node.clone().unwrap().borrow().action[best_move_idx]);
                    steps += 1;

                    let child = node.clone().unwrap().borrow().children[best_move_idx].clone().unwrap();
                    node = Some(child);
                }
            }

            // expand
            {
                let node_ = node.clone().unwrap();
                let rand_mv = node_.borrow_mut().untried_moves.pop();
                if let Some(mv_idx) = rand_mv {
                    state.borrow_mut().move_chess(node_.borrow().action[mv_idx]);
                    steps += 1;
                    let child = Some(Rc::new(RefCell::new(Node::new(
                            state.clone(), node.clone(),
                            node_.borrow().action[mv_idx]
                    ))));

                    node_.borrow_mut().children[mv_idx] = child.clone();

                    node = child;
                }
            }

            // rollout
            let mut rollout_step = 0;
            loop {
                let all_steps = state.borrow().generate_all_steps();
                if all_steps.is_empty() { break; }
                state.borrow_mut().move_chess(*all_steps.choose(&mut rand::thread_rng()).unwrap());
                rollout_step += 1;
            }

            // backpropagate
            let win_role = state.borrow().check_win();
            // println!("win_role: {:?} steps = {} rollout_step = {}", win_role, steps, rollout_step);

            for _ in 0..rollout_step {
                state.borrow_mut().undo_move();
            }

            let mut s = 0;
            while node.is_some() {
                let mut result = 0.0f32;
                if state.borrow().role != win_role {
                    result = 1.0 / (steps + rollout_step) as f32;
                }
                node.clone().unwrap().borrow_mut().update(result);
                node = node.unwrap().borrow_mut().parent.clone();
                if s < steps {
                    state.borrow_mut().undo_move();
                }
                s += 1;
            }
        }

        let mut action_idx = 0;
        let mut max_visited = 0.0;

        for (idx, c) in root.borrow().children.iter().enumerate() {
            if let Some(c) = c {
                let c = c.borrow();
                if c.visited > max_visited {
                    max_visited = c.visited;
                    action_idx = idx;
                }
                // println!("child({:?}) wins / visited = {} / {}", get_move(c.mv), c.wins, c.visited);
            }
        }

        let best_action = root.borrow().action[action_idx];
        // println!("best_action = {:?}", get_move(best_action));
        best_action
    }
}


impl Player for MCTSPlayer {
    fn get_move(&mut self) -> MOVE {
        self.mcts_run(500)
    }
}
