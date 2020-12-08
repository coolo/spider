use std::fs;
mod card;
mod deck;
mod moves;
mod pile;
use deck::Deck;
use moves::Move;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

fn visit(deck: Deck, visited: &mut BTreeMap<u64, i32>, orig_min_chaos: i32, level: u32) -> i32 {
    if level == 0 {
        return orig_min_chaos;
    }
    let mut min_chaos = orig_min_chaos;
    let hash = deck.hash(level);
    if let Some(chaos) = visited.get(&hash) {
        if *chaos < min_chaos {
            return *chaos;
        }
        return min_chaos;
    }

    let mut chaos = deck.chaos() as i32;
    if chaos == 0 {
        // special case for won
        chaos = 0 - level as i32;
    }
    if chaos < min_chaos {
        min_chaos = chaos;
    }
    //println!("Visit at level {} {}/{}", level, chaos, min_chaos);
    visited.insert(hash, chaos);
    let moves = deck.get_moves(true);
    for m in &moves {
        let newdeck = deck.apply_move(m);
        min_chaos = visit(newdeck, visited, min_chaos, level - 1);
    }
    min_chaos
}

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let mut deck = Deck::parse(&contents);
    let mut path = BTreeSet::new();

    println!("{}\n{}", deck.chaos(), deck.to_string());

    const MAX_CHAOS: i32 = 80000;
    let mut move_count = 0;
    loop {
        path.insert(deck.hash(0));
        let moves = deck.get_moves(false);
        let mut bestchaos = MAX_CHAOS;
        // invalid move
        let mut bestmove = Move::off(11, 0);
        for m in &moves {
            let mut visited = BTreeMap::new();
            deck.explain_move(m);
            let newdeck = deck.apply_move(m);
            if path.contains(&newdeck.hash(0)) {
                continue;
            }
            let newchaos = visit(newdeck, &mut visited, MAX_CHAOS, 20);
            println!("Visited in total: {} -> {}", visited.len(), newchaos);
            if newchaos < bestchaos {
                bestchaos = newchaos;
                bestmove = *m;
            }
        }
        if bestmove.from() == 11 {
            println!("No moves found {}", deck.chaos());
            if deck.chaos() == 0 {
                println!("WON!");
            }
            break;
        }
        deck.explain_move(&bestmove);
        deck = deck.apply_move(&bestmove);
        if !bestmove.is_off() {
            move_count += 1;
        }
        println!("{} {}\n{}", move_count, deck.chaos(), deck.to_string());
    }
}
