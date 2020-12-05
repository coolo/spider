mod card;
use card::Card;

fn main() {
    let c = Card::parse("|AH");
    println!("Card facedown {}", c.unwrap().faceup());
    let c = Card::parse("AH");
    println!("Card faceup {}", c.unwrap().faceup());
}
