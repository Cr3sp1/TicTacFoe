use crate::ai::Game;
use crate::game::Mark;

/// An AI that use Monte Carlo Tree search to play
#[derive(Clone, Debug, PartialEq)]
pub struct MCTSAi<T>
where
    T: Game + Clone,
{
    nodes: Vec<Node<T>>,
    pub ai_mark: Mark,
    enemy_mark: Mark,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node<T>
where
    T: Game + Clone,
{
    parent: Option<usize>,
    children: Vec<usize>,
    board: T,
}
