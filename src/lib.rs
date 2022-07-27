use std::marker::PhantomData;
use rand::Rng;

pub struct GameInfo {
    pub input_size: usize, 
    pub output_size: usize,
    pub players: usize
}

pub trait Game<Choice> where Choice: Clone {
    fn get_num_players(&self) -> usize;
    fn get_turn(&self) -> usize;

    fn get_choices(&self) -> Vec<Choice>;
    fn choose(&mut self, choice: &Choice);
    fn get_winner(&self) -> usize;
    fn random_play(&mut self) -> usize {
        let mut choices = self.get_choices();
        let mut rng = rand::thread_rng();

        while choices.len() > 0 {
            self.choose(&choices[rng.gen_range(0..choices.len())]);

            choices = self.get_choices();
        }

        self.get_winner()
    }
}

pub enum ExploitVsExplore {
    UCB1(f64),
    Random,
    ExploreFirst
}

impl ExploitVsExplore {
    fn get_func(&self) -> Box<dyn Fn(f64, f64, f64) -> f64> {
        match self {
            Self::UCB1(exploration_constant) => {
                let c = *exploration_constant;

                Box::new(move |wins, visits, parent_visits| {
                    wins / visits + c * (parent_visits.ln() / visits)
                })
            },
            Self::Random => Box::new(move |_, _, _| {
                let mut rng = rand::thread_rng();

                rng.gen::<f64>()
            }),
            Self::ExploreFirst => Box::new(move |_, visits, _| 1.0 / visits)
        }
    }
}

#[allow(dead_code)]
pub struct MCTS<Game_, Choice> where Choice: Clone, Game_: Game<Choice> + Clone {
    players: usize,
    exploit_vs_explore: ExploitVsExplore,
    __anoying: (PhantomData<Choice>, PhantomData<Game_>)
}

#[allow(dead_code)]
impl<Game_, Choice> MCTS<Game_, Choice> where Choice: Clone, Game_: Game<Choice> + Clone {
    pub fn new(initial_game_state: &Game_, exploit_vs_explore: ExploitVsExplore) -> Self {
        let players = initial_game_state.get_num_players();

        Self { players, exploit_vs_explore, __anoying: (PhantomData, PhantomData) }
    }
    
    // returns the winner
    fn mcts(&self, node: &mut Node<Game_, Choice>) -> usize {
        if node.visits == 0.0 {
            let winner = node.game_state.clone().random_play();

            node.next = node.game_state.get_choices().iter().map(|choice| {
                let mut next_game_state = node.game_state.clone();
                next_game_state.choose(choice);

                Node::new(next_game_state, Some(choice.clone()), self.players)
            }).collect();

            if node.next.len() == 0 {
                node.winner = Some(winner);
            }

            return node.update(winner, self.players);
        }

        if let Some(winner) = node.winner {
            return node.update(winner, self.players);
        }

        let next= node.best_next_index(
            node.game_state.get_turn(), 
            self.exploit_vs_explore.get_func()
        ).expect("Tried to branch on dead end node");

        let winner = self.mcts(&mut node.next[next]);

        node.update(winner, self.players)
    }

    pub fn advise(&self, game_state: &Game_, cycles: usize) -> Option<Choice> where Choice: PartialEq {
        let mut base_node = Node::default(game_state.clone(), self.players);

        self.advise_with_tree(&mut base_node, cycles)
    }

    pub fn advise_with_tree(&self, tree: &mut Node<Game_, Choice>, cycles: usize) -> Option<Choice> where Choice: PartialEq {
        for _ in 0..cycles {
            self.mcts(tree);
        }

        let choice = match tree.best_next_index(
            tree.game_state.get_turn(), 
            Box::new(|wins, visits, _| wins / visits)
        ) {
            None => None,
            Some(index) => {
                let choice = tree.next[index].choice.clone().expect("can't unwrap choice at best_next_index");
                tree.choose(&choice);

                Some(choice)
            }
        };

        choice
    }
}

pub struct Node<Game_, Choice> where Game_: Game<Choice> + Clone, Choice: Clone {
    winner: Option<usize>,
    pub game_state: Game_,
    choice: Option<Choice>,
    wins: Vec<f64>,
    visits: f64,
    next: Vec<Node<Game_, Choice>>
}

impl<Game_, Choice> Node<Game_, Choice> where Game_: Game<Choice> + Clone, Choice: Clone {
    fn new(game_state: Game_, choice: Option<Choice>, players: usize) -> Self {
        Node { winner: None, game_state, choice, wins: vec![0.0; players], visits: 0.0, next: Vec::new() }
    }

    pub fn default(game_state: Game_, players: usize) -> Self {
        Node { winner: None, game_state, choice: None, wins: vec![0.0; players], visits: 0.0, next: Vec::new() }
    }

    fn best_next_index(&self, player_id: usize, evaluator: Box<dyn Fn(f64, f64, f64)-> f64>) -> Option<usize> {
        if self.next.len() == 0 { return None };

        let mut best = (Vec::new(), -1.0);

        for i in 0..self.next.len() {
            let score = evaluator(self.next[i].wins[player_id - 1], self.next[i].visits + 0.00001, self.visits);

            if score > best.1 {
                best = (vec![i], score);
            } else if score == best.1 {
                best.0.push(i);
            }
        };
        
        let mut rng = rand::thread_rng();

        Some(best.0[rng.gen_range(0..best.0.len())])
    }

    fn update(&mut self, winner: usize, players: usize) -> usize {
        let bonus = if let Some(_) = self.winner { 1.0 } else { 1.0 };

        if winner > 0 { 
            self.wins[winner - 1] += bonus;
        } else { 
            self.wins.iter_mut().for_each(|win| *win += bonus / (players as f64)) 
        }

        self.visits += bonus;

        winner
    }

    fn choose(&mut self, choice: &Choice) where Choice: PartialEq {
        let chosen_node_index = self.next
            .iter()
            .position(|node| {
                if let Some(node_choice) = node.choice.clone() { &node_choice == choice } else { false }
            }).expect("The node does not include this choice");

        *self = self.next.remove(chosen_node_index);
    }
}

impl<Game_, Choice> std::fmt::Display for Node<Game_, Choice> where Game_: Game<Choice> + Clone, Choice: Clone + std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.next.len() == 0 && self.visits == 0.0 {
            return write!(f, "{{}}");
        } else if let Some(winner) = self.winner { 
            return write!(f, "{{\"choice\": \"{:?}\",  \"winner\": {}, \"visits\": {}}}", self.choice, winner, self.visits);
        };

        let mut str = format!("[{}", self.next[0]);

        for node in self.next[1..].iter() {
            str += format!(", {}", node).as_str();
        }

        str += "]";

        write!(f, "{{\"choice\": \"{:?}\", \"wins\": {:?}, \"visits\": {}, \"next\": {}}}", self.choice, self.wins, self.visits, str)
    }
}