use std::fs;
mod card;
mod deck;
mod moves;
mod pile;
use card::Card;
use deck::Deck;
use std::fs::File;
use std::io;
use std::io::Write;

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let mut cap: usize = 5000;
    if let Some(ncap) = std::env::args().nth(2) {
        cap = ncap.parse().expect("Integer");
    }
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let mut deck = Deck::parse(&contents);
    deck.shuffle_unknowns(2);

    let result = deck.shortest_path(cap, 50_000_000);
    if result.is_none() {
        return;
    }
    let result = result.unwrap();
    println!("{}", result);
    let mut mc = 0;
    let mut orig = deck.clone();
    for m in deck.win_moves() {
        if !m.is_off() {
            mc += 1;
        }
        print!("Move {}: ", mc);
        orig.explain_move(&m);
        orig = orig.apply_move(&m);

        if orig.top_card_unknown(m.from()) {
            println!("What's up?");
            let stdin = io::stdin();
            let buffer = &mut String::new();

            stdin.read_line(buffer).expect("read");
            let c = Card::parse(&buffer.trim()).expect("valid card");

            orig.replace_play_card(m.from(), m.index() - 1, &c);
            println!("{}", orig.to_string());

            // Open a file in write-only mode, returns `io::Result<File>`
            let mut file = match File::create("tmp") {
                Err(why) => panic!("couldn't create tmp: {}", why),
                Ok(file) => file,
            };

            match file.write_all(orig.to_string().as_bytes()) {
                Err(why) => panic!("couldn't write to tmp {}", why),
                Ok(_) => println!("successfully wrote to tmp"),
            }
            std::process::exit(1);
        }
    }
}
