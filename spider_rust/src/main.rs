use std::fs;
mod card;
mod deck;
mod pile;
use deck::Deck;
use pile::Pile;
use std::collections::HashMap;

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    // need to make this implicit
    let mut pilemap: HashMap<u64, Pile> = HashMap::new();
    let deck = Deck::parse(&contents, &mut pilemap);
    println!("{}", deck.to_string(&pilemap));
}
