use crate::ai::{Game, Move};
use crate::game::{GameState, Mark};
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::f32::consts::SQRT_2;

const N_ROUNDS: i16 = 1000;

// TO Do
// Use ai_mark

/// An AI that use Monte Carlo Tree search to play
#[derive(Clone, Debug, PartialEq)]
pub struct MCTSAi<T>
where
    T: Game + Clone,
{
    nodes: Vec<Node<T>>,
    root_id: usize,
    pub ai_mark: Mark,
}

impl<T> MCTSAi<T>
where
    T: Game + Clone + PartialEq,
{
    pub fn new(board: T, ai_mark: Mark) -> Self {
        Self {
            nodes: vec![Node::new(board, Mark::X, None)],
            root_id: 0,
            ai_mark,
        }
    }

    pub fn choose_move(&mut self, board: &T) -> Move {
        let new_root_id = self.find_state(board);
        self.reroot(new_root_id);

        // Explore and expand tree
        for _ in 0..N_ROUNDS {
            let selected_id = self.selection();
            let result = self.simulate(selected_id);
            self.back_propagate(selected_id, result);
        }

        // Find move with the best probability of winning
        let mut best_move_id = 0;
        let mut best_chance = 0.0;
        let children_ids = self.nodes[self.root_id].children.as_ref().unwrap();
        for (i, child_id) in children_ids.iter().enumerate() {
            let winning_chance = self.nodes[*child_id].winning_chance(self.ai_mark);
            if winning_chance > best_chance {
                best_chance = winning_chance;
                best_move_id = i;
            }
        }

        if self.nodes[self.root_id].possible_moves.is_empty() {
            panic!("No available moves found by MCTSAi");
        }

        let best_move = self.nodes[self.root_id].possible_moves[best_move_id];
        self.reroot(self.nodes[self.root_id].children.as_ref().unwrap()[best_move_id]);
        best_move
    }

    fn selection(&mut self) -> usize {
        let mut starting_node = self.root_id;
        loop {
            // Select if unexplored
            let total_plays = self.nodes[starting_node].plays;
            if total_plays == 0.0 {
                return starting_node;
            }

            if self.nodes[starting_node].children.is_none() {
                self.make_children(starting_node);
            }

            let children = self.nodes[starting_node].children.as_ref().unwrap();
            // Select if it has no children
            if children.len() == 0 {
                return starting_node;
            }

            let mut best_potential = -1.0;
            let mut best_child = 0;
            for child_id in children.iter() {
                // Select the first unexplored child if any
                if self.nodes[*child_id].plays == 0.0 {
                    return *child_id;
                }

                // Find child with best potential
                let child_potential = self.nodes[*child_id].potential(total_plays - 1.0);
                if child_potential > best_potential {
                    best_potential = child_potential;
                    best_child = *child_id;
                }
            }

            // Restart from child node with best potential
            starting_node = best_child;
        }
    }

    fn simulate(&mut self, node_id: usize) -> GameState {
        let mut active_player = self.nodes[node_id].active_player;
        let mut board = self.nodes[node_id].board.clone();
        while board.get_state() == GameState::Playing {
            // Select a move at random
            let possible_moves = board.get_possible_moves();
            let mut rng = rand::rng();
            let mv = possible_moves.choose(&mut rng).unwrap();

            board.play(mv, active_player);
            active_player = active_player.switch();
        }

        board.get_state()
    }

    fn back_propagate(&mut self, from_id: usize, result: GameState) {
        let mut node_id = from_id;
        loop {
            self.nodes[node_id].plays += 1.0;
            match result {
                GameState::Won(mark) => {
                    if mark == self.nodes[node_id].active_player {
                        self.nodes[node_id].wins += 1.0;
                    }
                }
                _ => self.nodes[node_id].wins += 0.5,
            }

            match self.nodes[node_id].parent {
                Some(parent_id) => node_id = parent_id,
                None => return,
            }
        }
    }

    fn make_children(&mut self, parent_id: usize) {
        let mut children: Vec<Node<T>> = Vec::new();

        for mv in self.nodes[parent_id].possible_moves.iter() {
            let mut new_board = self.nodes[parent_id].board.clone();
            new_board.play(mv, self.nodes[parent_id].active_player);
            children.push(Node::new(
                new_board,
                self.nodes[parent_id].active_player.switch(),
                Some(parent_id),
            ));
        }

        let children_ids = (self.nodes.len()..self.nodes.len() + children.len()).collect();
        self.nodes[parent_id].children = Some(children_ids);
        self.nodes.append(&mut children);
    }

    fn reroot(&mut self, new_root_id: usize) {
        self.nodes[self.root_id].parent = None;
        self.root_id = new_root_id;
    }

    fn find_state(&mut self, board: &T) -> usize {
        if self.nodes[self.root_id].board == *board {
            return self.root_id;
        }

        if self.nodes[self.root_id].children.is_none() {
            self.make_children(self.root_id);
        }
        for child_id in self.nodes[self.root_id].children.as_ref().unwrap().iter() {
            if self.nodes[*child_id].board == *board {
                return *child_id;
            }
        }
        panic!("Board state following move was not found in MCTS.");
    }

    pub fn switch_starting_mark(&mut self) {
        if self.nodes.len() != 1 {
            panic!("Starting mark must be switched before any tree exploration.");
        }
        self.nodes[0].active_player = self.nodes[0].active_player.switch();
    }

    pub fn reset(&mut self) {
        let clean_board = self.nodes[0].board.clone();
        self.nodes = vec![Node::new(clean_board, Mark::X, None)];
        self.root_id = 0;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node<T>
where
    T: Game + Clone,
{
    parent: Option<usize>,
    children: Option<Vec<usize>>,
    board: T,
    active_player: Mark,
    wins: f32,
    plays: f32,
    possible_moves: Vec<Move>,
}

impl<T> Node<T>
where
    T: Game + Clone,
{
    pub fn new(board: T, active_player: Mark, parent: Option<usize>) -> Self {
        let mut possible_moves = board.get_possible_moves();
        let mut rng = rand::rng();
        possible_moves.shuffle(&mut rng);
        Node {
            parent,
            children: None,
            board,
            active_player,
            wins: 0.0,
            plays: 0.0,
            possible_moves,
        }
    }

    fn potential(&self, total_plays: f32) -> f32 {
        (self.plays - self.wins) / total_plays + SQRT_2 * (total_plays.ln() / self.plays).sqrt()
    }

    fn winning_chance(&self, ai_mark: Mark) -> f32 {
        if self.plays <= 0.0 {
            0.0
        } else {
            if ai_mark == self.active_player {
                return self.wins / self.plays;
            }
            1.0 - self.wins / self.plays
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_choose_moves() {}
// }
