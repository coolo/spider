use std::fs;
mod card;
mod deck;
mod moves;
mod pile;
use card::Card;
use clap::{App, Arg};
use deck::Deck;
use std::fs::File;
use std::io;
use std::io::Write;

fn play_one_round(filename: &str, cap: usize, suits: usize, orig_filename: Option<&str>) -> bool {
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let mut deck = Deck::parse(&contents);
    deck.shuffle_unknowns(suits);

    let result = deck.shortest_path(cap, 50_000_000);
    if result.is_none() {
        return false;
    }
    let mut mc = 0;
    let mut orig = deck.clone();
    orig.reset_moves();
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

            let mut file = match File::create("tmp") {
                Err(why) => panic!("couldn't create tmp: {}", why),
                Ok(file) => file,
            };

            match file.write_all(orig.to_string().as_bytes()) {
                Err(why) => panic!("couldn't write to tmp {}", why),
                Ok(_) => println!("successfully wrote to tmp"),
            }

            let filename = orig_filename.expect("filename");
            let contents =
                fs::read_to_string(filename).expect("Something went wrong reading the file");
            let mut deck2 = Deck::parse(&contents);
            deck2.replace_play_card(m.from(), m.index() - 1, &c);
            let mut file = match File::create(filename) {
                Err(why) => panic!("couldn't create tmp: {}", why),
                Ok(file) => file,
            };

            match file.write_all(deck2.to_string().as_bytes()) {
                Err(why) => panic!("couldn't write to tmp {}", why),
                Ok(_) => println!("successfully wrote to tmp"),
            }

            return true;
        }
    }
    false
}

fn main() {
    let matches = App::new("spider")
        .version("1.0")
        .arg(
            Arg::with_name("orig")
                .long("orig")
                .takes_value(true)
                .help("Original file name"),
        )
        .arg(
            Arg::with_name("filename")
                .takes_value(true)
                .help("Temporary file name"),
        )
        .arg(
            Arg::with_name("cap")
                .long("cap")
                .takes_value(true)
                .help("Runtime cap"),
        )
        .arg(
            Arg::with_name("suits")
                .long("suits")
                .takes_value(true)
                .default_value("2")
                .help("Number of suits"),
        )
        .get_matches();

    let filename = matches.value_of("filename").expect("filename");
    let mut cap: usize = 5000;
    if let Some(ncap) = matches.value_of("cap") {
        cap = ncap.parse().expect("Integer");
    }
    loop {
        if !play_one_round(
            filename,
            cap,
            matches.value_of("suits").unwrap().parse().unwrap(),
            matches.value_of("orig"),
        ) {
            break;
        }
    }
}
