# Monte
Implement mcts (Monte Carlo tree search) into an arbitrary game

Impliment this trait
```rust
pub trait Game<Choice> where Choice: Clone {
    fn get_num_players(&self) -> usize;
    fn get_turn(&self) -> usize;

    fn get_choices(&self) -> Vec<Choice>;
    fn choose(&mut self, choice: &Choice);
    fn get_winner(&self) -> usize;
}
```
