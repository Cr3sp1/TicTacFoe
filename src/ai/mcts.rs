use crate::ai::{Game, Move};
use crate::game::{GameState, Mark};
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::f32::consts::SQRT_2;

/// Number of MCTS simulation rounds per move decision.
const N_ROUNDS: i16 = 1000;

/// An AI opponent that uses Monte Carlo Tree Search (MCTS) to select moves.
///
/// MCTS builds a game tree incrementally by repeatedly running random
/// playouts from selected nodes, then back-propagating results to update
/// win/play statistics. The tree is reused across turns to amortize cost.
#[derive(Clone, Debug, PartialEq)]
pub struct MCTSAi<T>
where
    T: Game + Clone,
{
    /// Arena-allocated tree nodes, indexed by `usize` IDs.
    nodes: Vec<Node<T>>,
    /// Index of the current root node (represents the present board state).
    root_id: usize,
    /// The mark (`X` or `O`) this AI plays as.
    pub ai_mark: Mark,
}

impl<T> MCTSAi<T>
where
    T: Game + Clone + PartialEq,
{
    /// Creates a new `MCTSAi` with the given starting board and mark.
    ///
    /// The tree is initialized with a single root node for `board`.
    /// The first active player is always assumed to be `Mark::X`.
    pub fn new(board: T, ai_mark: Mark) -> Self {
        Self {
            nodes: vec![Node::new(board, Mark::X, None)],
            root_id: 0,
            ai_mark,
        }
    }

    /// Chooses the best move for the current `board` state.
    ///
    /// Runs [`N_ROUNDS`] of selection → simulation → back-propagation, then
    /// returns the child move with the highest estimated winning chance.
    /// The tree root is advanced to the chosen child for future reuse.
    ///
    /// # Panics
    /// Panics if there are no legal moves available.
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

    /// Selects a node to simulate using the UCB1 policy.
    ///
    /// Traverses from the root, preferring unexplored nodes first, then
    /// the child with the highest UCB1 potential score.
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
            // Select if it has no children (terminal node)
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

    /// Runs a random playout from `node_id` to a terminal state.
    ///
    /// Returns the [`GameState`] at the end of the playout.
    fn simulate(&mut self, node_id: usize) -> GameState {
        let mut active_player = self.nodes[node_id].active_player;
        let mut board = self.nodes[node_id].board.clone();
        while board.get_state() == GameState::Playing {
            let possible_moves = board.get_possible_moves();
            let mut rng = rand::rng();
            let mv = possible_moves.choose(&mut rng).unwrap();
            board.play(mv, active_player);
            active_player = active_player.switch();
        }
        board.get_state()
    }

    /// Propagates a playout `result` from `from_id` up to the root.
    ///
    /// Each ancestor's `plays` is incremented; `wins` is incremented by
    /// `1.0` on a win for the node's active player, or `0.5` on a draw.
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

    /// Expands `parent_id` by creating one child node per legal move.
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

    /// Sets `new_root_id` as the new root, detaching it from its parent.
    fn reroot(&mut self, new_root_id: usize) {
        self.nodes[self.root_id].parent = None;
        self.root_id = new_root_id;
    }

    /// Locates the node matching `board` among the root or its direct children.
    ///
    /// # Panics
    /// Panics if the board state is not found (indicates an illegal board transition).
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

    /// Swaps the root node's active player.
    ///
    /// Must be called before any tree exploration (i.e. when the tree has
    /// exactly one node). Use this when the AI plays as `O` and moves second.
    ///
    /// # Panics
    /// Panics if the tree has already been expanded.
    pub fn switch_starting_mark(&mut self) {
        if self.nodes.len() != 1 {
            panic!("Starting mark must be switched before any tree expansion.");
        }
        self.nodes[0].active_player = self.nodes[0].active_player.switch();
    }

    /// Resets the tree to its initial single-node state, discarding all exploration.
    pub fn reset(&mut self) {
        let clean_board = self.nodes[0].board.clone();
        self.nodes = vec![Node::new(clean_board, Mark::X, None)];
        self.root_id = 0;
    }
}

/// A single node in the MCTS tree.
#[derive(Clone, Debug, PartialEq)]
pub struct Node<T>
where
    T: Game + Clone,
{
    parent: Option<usize>,
    children: Option<Vec<usize>>,
    board: T,
    /// The player whose turn it is at this node.
    active_player: Mark,
    wins: f32,
    plays: f32,
    /// Legal moves from this node's board state (shuffled for exploration variety).
    possible_moves: Vec<Move>,
}

impl<T> Node<T>
where
    T: Game + Clone,
{
    /// Creates a new leaf node for `board` with `active_player` to move.
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

    /// UCB1 score used to balance exploration vs. exploitation during selection.
    fn potential(&self, total_plays: f32) -> f32 {
        (self.plays - self.wins) / total_plays + SQRT_2 * (total_plays.ln() / self.plays).sqrt()
    }

    /// Estimated probability that `ai_mark` wins from this node, based on recorded playouts.
    ///
    /// Returns `0.0` if the node has not been visited yet.
    fn winning_chance(&self, ai_mark: Mark) -> f32 {
        if self.plays <= 0.0 {
            return 0.0;
        }
        if ai_mark == self.active_player {
            self.wins / self.plays
        } else {
            1.0 - self.wins / self.plays
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::base::SmallBoard;

    fn make_ai(mark: Mark) -> MCTSAi<SmallBoard> {
        MCTSAi::new(SmallBoard::new(), mark)
    }

    #[test]
    fn test_new_creates_single_root_node() {
        let ai = make_ai(Mark::X);
        assert_eq!(ai.nodes.len(), 1);
        assert_eq!(ai.root_id, 0);
        assert_eq!(ai.ai_mark, Mark::X);
    }

    #[test]
    fn test_switch_starting_mark_toggles_active_player() {
        let mut ai = make_ai(Mark::O);
        assert_eq!(ai.nodes[0].active_player, Mark::X);
        ai.switch_starting_mark();
        assert_eq!(ai.nodes[0].active_player, Mark::O);
    }

    #[test]
    #[should_panic(expected = "Starting mark must be switched before any tree expansion.")]
    fn test_switch_starting_mark_panics_after_expansion() {
        let mut ai = make_ai(Mark::X);
        // Trigger expansion
        for _ in 0..2 {
            let selected_id = ai.selection();
            let result = ai.simulate(selected_id);
            ai.back_propagate(selected_id, result);
        }
        ai.switch_starting_mark();
    }

    #[test]
    fn test_reset_collapses_tree_to_single_node() {
        let mut ai = make_ai(Mark::X);
        // Expand the tree with a few rounds
        for _ in 0..10 {
            let id = ai.selection();
            let result = ai.simulate(id);
            ai.back_propagate(id, result);
        }
        assert!(ai.nodes.len() > 1, "Tree should have expanded");
        ai.reset();
        assert_eq!(ai.nodes.len(), 1);
        assert_eq!(ai.root_id, 0);
        assert_eq!(ai.nodes[0].plays, 0.0);
    }

    #[test]
    fn test_choose_move_returns_valid_move_on_empty_board() {
        let board = SmallBoard::new();
        let mut ai = make_ai(Mark::X);
        let mv = ai.choose_move(&board);
        let (row, col) = mv.unwrap_base();
        assert!(row < 3 && col < 3);
    }

    #[test]
    fn test_winning_chance_unvisited_node_is_zero() {
        let node = Node::new(SmallBoard::new(), Mark::X, None);
        assert_eq!(node.winning_chance(Mark::X), 0.0);
    }

    #[test]
    fn test_winning_chance_active_player_perspective() {
        let mut node = Node::new(SmallBoard::new(), Mark::X, None);
        node.plays = 4.0;
        node.wins = 3.0;
        // Active player is X, asking for X → wins/plays = 0.75
        assert!((node.winning_chance(Mark::X) - 0.75).abs() < 1e-6);
        // Asking for O (opponent) → 1 - 0.75 = 0.25
        assert!((node.winning_chance(Mark::O) - 0.25).abs() < 1e-6);
    }

    // --- back_propagate ---

    #[test]
    fn test_back_propagate_increments_plays() {
        let mut ai = make_ai(Mark::X);
        ai.back_propagate(0, GameState::Draw);
        assert_eq!(ai.nodes[0].plays, 1.0);
    }

    #[test]
    fn test_back_propagate_draw_adds_half_win() {
        let mut ai = make_ai(Mark::X);
        ai.back_propagate(0, GameState::Draw);
        assert!((ai.nodes[0].wins - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_back_propagate_win_credits_correct_player() {
        let mut ai = make_ai(Mark::X);
        // Root active_player starts as X
        ai.back_propagate(0, GameState::Won(Mark::X));
        assert!((ai.nodes[0].wins - 1.0).abs() < 1e-6);

        // A win for O should not credit root (active X)
        ai.back_propagate(0, GameState::Won(Mark::O));
        // wins should still be 1.0 (not incremented again)
        assert!((ai.nodes[0].wins - 1.0).abs() < 1e-6);
    }
}
