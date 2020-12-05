use std::fs;
mod card;
mod deck;
mod pile;
use deck::Deck;
use pile::Pile;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::VecDeque;

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    // need to make this implicit
    let mut pilemap: HashMap<u64, Pile> = HashMap::new();
    let deck = Deck::parse(&contents, &mut pilemap);

    let mut unvisted: VecDeque<Deck> = VecDeque::new();
    unvisted.push_back(deck);
    let mut visited = BTreeSet::new();
    visited.insert(deck.hash());

    loop {
        match unvisted.pop_front() {
            None => break,
            Some(deck) => {
                //println!("{}", deck.to_string(&pilemap));
                println!(
                    "Visited: {} unvisited: {} piles: {}",
                    visited.len(),
                    unvisted.len(),
                    pilemap.len()
                );
                let moves = deck.get_moves(&pilemap);
                for m in &moves {
                    //deck.explain_move(m, &pilemap);
                    let newdeck = deck.apply_move(m, &mut pilemap);
                    let hash = newdeck.hash();
                    if !visited.contains(&hash) {
                        visited.insert(hash);
                        unvisted.push_back(newdeck);
                    }
                }
            }
        }
    }
}
