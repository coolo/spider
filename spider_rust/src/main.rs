use std::fs;
mod card;
mod deck;
mod moves;
mod pile;
use card::Card;
use clap::{App, Arg};
use deck::Deck;
use deck::DeltaMove;
use pile::Pile;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::fs::File;
use std::io;
use std::io::Write;

fn generate_deck(filename: &str) {
    let mut deck = Deck::empty();
    for i in 0..10 {
        print!("Top card for Pile {}? ", i + 1);
        io::stdout().flush().unwrap();
        let buffer = &mut String::new();
        io::stdin().read_line(buffer).expect("read");
        let c = Card::parse(&buffer.trim()).expect("valid card");
        let mut pile_str = String::from("|XX |XX |XX |XX ");
        if i < 4 {
            pile_str += "|XX ";
        }
        pile_str += &c.to_string();
        let pile = Pile::parse(&pile_str).expect("valid pile");
        deck.set_play(i, pile);
        println!("{}", deck.to_string());
    }
    for i in 0..5 {
        print!("Cards for Talon {}? ", i + 1);
        io::stdout().flush().unwrap();
        let buffer = &mut String::new();
        io::stdin().read_line(buffer).expect("read");
        let pile = Pile::parse(&buffer.trim()).expect("valid pile");
        if pile.count() != 10 {
            panic!("Need 10 cards")
        }
        deck.set_talon(i, pile);
        println!("{}", deck.to_string());
    }
    let mut file = match File::create(filename) {
        Err(why) => panic!("couldn't create file {}: {}", filename, why),
        Ok(file) => file,
    };

    match file.write_all(deck.to_string().as_bytes()) {
        Err(why) => panic!("couldn't write to {} {}", filename, why),
        Ok(_) => println!("successfully wrote to {}", filename),
    }
}

fn play_one_round(
    filename: &str,
    cap: usize,
    suits: usize,
    orig_filename: Option<&str>,
    debug: bool,
    yaml: bool,
) -> bool {
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let mut deck = Deck::parse(&contents);
    deck.shuffle_unknowns(suits);

    let result = deck.shortest_path(cap, debug, None);
    if result.is_none() {
        println!("No win");
        return false;
    }
    let mut won_decks: HashSet<u64> = HashSet::new();
    let mut mc = 0;
    let mut orig = deck.clone();
    orig.reset_moves();
    if yaml {
        println!("moves:");
    }
    for m in deck.win_moves() {
        won_decks.insert(orig.hash());
        if !m.is_off() {
            mc += 1;
        }
        if yaml {
            println!("  - from: {}", m.from());
            println!("    to: {}", m.to());
            println!("    index: {}", m.index());
            if m.is_off() {
                println!("    off: true");
            }
            if m.is_talon() {
                println!("    talon: true");
            }
            println!("    number: {}", mc);
        } else {
            print!("Move {}: {} ", mc, orig.explain_move(&m));
            deck = deck.apply_move(&m);
        }
        let dm = DeltaMove::new(&orig, &m);

        let newdeck = orig.apply_move(&m);
        if !yaml {
            println!(
                " (Chaos {} Playable {} Off {} Free {} Talons {} Under {})",
                newdeck.chaos(),
                newdeck.playable(),
                newdeck.in_off(),
                newdeck.free_plays(),
                newdeck.free_talons(),
                newdeck.under()
            );
        }
        if !dm.any_good() {
            println!("Not a good move:\n{} {:?}", orig.to_string(), dm);
        }
        orig = newdeck;
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

            if orig_filename.is_some() {
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
                    Err(why) => panic!("couldn't write to {} {}", filename, why),
                    Ok(_) => println!("successfully wrote to {}", filename),
                }
            }

            return true;
        }
    }
    if debug {
        deck.reset_moves();
        deck.shortest_path(cap, debug, Some(won_decks));
    }
    false
}

struct WeightedDeck {
    deck: Deck,
    depth: u32,
    moves: u32,
    total: u32,
    hash: u64,
}

impl PartialOrd for WeightedDeck {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WeightedDeck {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .total
            .cmp(&self.total)
            .then(self.depth.cmp(&other.depth))
    }
}

impl PartialEq for WeightedDeck {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for WeightedDeck {}

fn pick(
    heap: &mut BinaryHeap<WeightedDeck>,
    seen: &mut HashSet<u64>,
    cap: usize,
    best_before: usize,
    orig: &Deck,
) -> usize {
    let wdeck = heap.pop();
    if wdeck.is_none() {
        return 0;
    }
    let wdeck = wdeck.unwrap();
    let depth = wdeck.depth;
    print!("Picked {}+{}={} (", depth, wdeck.moves, wdeck.total);

    let deck = wdeck.deck;
    if deck.is_won() {
        println!("WON");
        return 0;
    }
    let mut moves = vec![];
    deck.get_moves(&mut moves);
    let mut best_total = deck::MAX_MOVES;
    let mut bestdeck: Option<Deck> = None;

    for m in &moves {
        //deck.explain_move(&m);
        let mut newdeck = deck.apply_move(m);
        let hash = newdeck.hash();
        if seen.contains(&hash) {
            continue;
        }
        seen.insert(hash);
        //println!("New\n{}", newdeck.to_string());
        let orig_move_index = newdeck.get_moves_index();
        let won = newdeck.shortest_path(cap, false, None);

        if won.is_none() || won.unwrap() < 0 {
            //println!("Move didn't win");
        } else {
            let won = won.unwrap() as u32;
            //println!("Move gave {}", won);
            print!("{} ", depth + won + 1);
            if ((won + depth + 1) as usize) < best_total {
                best_total = (won + depth + 1) as usize;
                if best_total < best_before {
                    bestdeck = Some(newdeck.clone());
                }
            }
            newdeck.set_moves_index(orig_move_index);
            heap.push(WeightedDeck {
                deck: newdeck,
                hash: hash,
                depth: depth + 1,
                moves: won as u32,
                total: won + depth + 1,
            });
        }
    }
    println!(")");
    if best_total > wdeck.total as usize {
        println!("There was a slip!");
        let filename = format!("slip.{}-{}", best_total, wdeck.total);
        let mut file = match File::create(&filename) {
            Err(why) => panic!("couldn't create tmp: {}", why),
            Ok(file) => file,
        };

        match file.write_all(deck.to_string().as_bytes()) {
            Err(why) => panic!("couldn't write to {} {}", filename, why),
            Ok(_) => println!("successfully wrote to {}", filename),
        }
    }
    if let Some(deck) = bestdeck {
        let moves = deck.win_moves();
        let mut deck = orig.clone();
        let mut mc = 0;
        for m in moves {
            if !m.is_off() {
                mc += 1;
            }
            print!("Move {}: {}", mc, deck.explain_move(&m));
            deck = deck.apply_move(&m);
            println!(" (Chaos {} Playable {})", deck.chaos(), deck.playable());
        }
    }
    best_total
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
                .multiple(false)
                .required(true)
                .help("Temporary file name"),
        )
        .arg(
            Arg::with_name("yaml")
                .long("yaml")
                .help("Output moves as yaml"),
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
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .help("Output progress"),
        )
        .arg(
            Arg::with_name("generate")
                .long("generate")
                .help("Generate a new deck file"),
        )
        .arg(
            Arg::with_name("slow")
                .long("slow")
                .help("Use A+ to search further"),
        )
        .get_matches();

    let filename = matches.value_of("filename").expect("filename");
    if matches.is_present("generate") {
        generate_deck(filename);
        return;
    }
    let mut cap: usize = 200;
    if let Some(ncap) = matches.value_of("cap") {
        cap = ncap.parse().expect("Integer");
    }

    let suits = matches.value_of("suits").unwrap().parse().unwrap();

    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let mut deck = Deck::parse(&contents);
    deck.shuffle_unknowns(suits);

    if matches.is_present("slow") {
        let mut heap: BinaryHeap<WeightedDeck> = BinaryHeap::new();
        let mc = deck.shortest_path(cap, false, None).unwrap();
        deck.reset_moves();
        assert!(mc > 0);
        heap.push(WeightedDeck {
            hash: deck.hash(),
            deck: deck.clone(),
            depth: 0,
            moves: mc as u32,
            total: mc as u32,
        });
        let mut seen = HashSet::new();
        let mut best = deck::MAX_MOVES;

        loop {
            let current_best = pick(&mut heap, &mut seen, cap, best, &deck);
            if current_best == 0 {
                break;
            }
            if best > current_best {
                best = current_best;
            }
        }
    } else {
        loop {
            if !play_one_round(
                filename,
                cap,
                suits,
                matches.value_of("orig"),
                matches.is_present("debug"),
                matches.is_present("yaml"),
            ) {
                break;
            }
        }
    }
}
