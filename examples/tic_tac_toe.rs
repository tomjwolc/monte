extern crate monte;

use monte::*;

const WINS: [[usize; 3]; 8] = [
    // rows
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    // columns
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    // diagnols
    [0, 4, 8],
    [2, 4, 6]
];

#[derive(Clone)]
struct TTT {
    board: [usize; 9],
    turn: usize
}

impl TTT {
    fn new() -> Self {
        Self { board: [0; 9], turn: 1 }
    }
}

impl Game<usize> for TTT {
    fn get_num_players(&self) -> usize {
        2
    }

    fn get_turn(&self) -> usize {
        self.turn
    }

    fn get_choices(&self) -> Vec<usize> {
        let mut choices = Vec::new();

        if self.get_winner() != 0 { return choices };

        for i in 0..9 {
            if self.board[i] == 0 { choices.push(i) }
        }

        choices
    }

    fn choose(&mut self, choice: &usize) {
        self.board[*choice] = self.turn;
        self.turn = if self.turn == 1 { 2 } else { 1 };
    }

    fn get_winner(&self) -> usize {
        let mut winner = 0;

        if WINS.iter().any(|win| {
            winner = self.board[win[0]];

            self.board[win[0]] == self.board[win[1]] && 
            self.board[win[1]] == self.board[win[2]] && 
            self.board[win[0]] != 0
        }) {
            winner
        } else {
            0
        }
    }
}

impl std::fmt::Display for TTT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b: Vec<&str> = self.board.iter().map(|player_id| {
            match player_id {
                1 => "X",
                2 => "O",
                _ => " "
            }
        }).collect();


        write!(f, "
              {} | {} | {}
            -------------
              {} | {} | {}
            -------------
              {} | {} | {}
        ", b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8])
    }
}

fn main() {
    let ttt = TTT::new();

    let mcts = MCTS::new(&ttt, ExploitVsExplore::UCB1(1.41));

    for i in 0..1 {
        println!("Tree-less game #{}", i + 1);

        play_ttt_without_tree(&ttt, &mcts);
    }

    for i in 0..1 {
        println!("Tree game #{}", i + 1);

        play_ttt_with_tree(&ttt, &mcts);
    }
}

fn play_ttt_without_tree(ttt: &TTT, mcts: &MCTS<TTT, usize>) {
    let mut clone = ttt.clone();
    println!("{}", clone);

    while let Some(choice) = mcts.advise(&clone, 1000) {
        clone.choose(&choice);
        println!("{}", clone);
    }

    println!("Winner: {}", clone.get_winner())
}

fn play_ttt_with_tree(ttt: &TTT, mcts: &MCTS<TTT, usize>) {
    let mut clone = ttt.clone();
    let mut tree = monte::Node::default(clone.clone(), clone.get_num_players());
    println!("{} \n{}", tree, clone);

    while let Some(choice) = mcts.advise_with_tree(&mut tree, 1000) {
        clone.choose(&choice);
        println!("{} \n{}", tree, clone);
    }

    println!("Winner: {}", clone.get_winner())
}