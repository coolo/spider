use std::fs;
mod card;
mod deck;
mod moves;
mod pile;
use deck::Deck;
use moves::Move;
use pile::PileManager;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

fn visit(
    deck: Deck,
    pilemap: &mut PileManager,
    visited: &mut BTreeMap<u64, i32>,
    orig_min_chaos: i32,
    level: u32,
) -> i32 {
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

    let mut chaos = deck.chaos(&pilemap) as i32;
    if chaos == 0 {
        // special case for won
        chaos = 0 - level as i32;
    }
    if chaos < min_chaos {
        min_chaos = chaos;
    }
    //println!("Visit at level {} {}/{}", level, chaos, min_chaos);
    visited.insert(hash, chaos);
    let moves = deck.get_moves(&pilemap, true);
    for m in &moves {
        let newdeck = deck.apply_move(m, pilemap);
        min_chaos = visit(newdeck, pilemap, visited, min_chaos, level - 1);
    }
    min_chaos
}

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    // need to make this implicit
    let mut pilemap = PileManager::new();
    let mut deck = Deck::parse(&contents, &mut pilemap);
    let mut path = BTreeSet::new();

    println!("{}\n{}", deck.chaos(&pilemap), deck.to_string(&pilemap));

    const MAX_CHAOS: i32 = 80000;
    let mut move_count = 0;
    loop {
        path.insert(deck.hash(0));
        let moves = deck.get_moves(&pilemap, false);
        let mut bestchaos = MAX_CHAOS;
        // invalid move
        let mut bestmove = Move::off(11, 0);
        for m in &moves {
            let mut visited = BTreeMap::new();
            deck.explain_move(m, &pilemap);
            let newdeck = deck.apply_move(m, &mut pilemap);
            if path.contains(&newdeck.hash(0)) {
                continue;
            }
            let newchaos = visit(newdeck, &mut pilemap, &mut visited, MAX_CHAOS, 20);
            println!("Visited in total: {} -> {}", visited.len(), newchaos);
            if newchaos < bestchaos {
                bestchaos = newchaos;
                bestmove = *m;
            }
        }
        if bestmove.from() == 11 {
            println!("No moves found {}", deck.chaos(&pilemap));
            if deck.chaos(&pilemap) == 0 {
                println!("WON!");
            }
            break;
        }
        deck.explain_move(&bestmove, &pilemap);
        deck = deck.apply_move(&bestmove, &mut pilemap);
        if !bestmove.is_off() {
            move_count += 1;
        }
        println!(
            "{} {}\n{}",
            move_count,
            deck.chaos(&pilemap),
            deck.to_string(&pilemap)
        );
    }
}
