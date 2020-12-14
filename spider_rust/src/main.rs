use std::fs;
mod card;
mod deck;
mod moves;
mod pile;
use deck::Deck;

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let mut cap: usize = 5000;
    if let Some(ncap) = std::env::args().nth(2) {
        cap = ncap.parse().expect("Integer");
    }
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let mut deck = Deck::parse(&contents);
    deck.shuffle_unknowns(2);

    println!("{}", deck.shortest_path(cap, 50_000_000).expect("win"));
}
