use std::fs;
mod card;
mod deck;
mod moves;
mod pile;
use deck::Deck;
use moves::Move;
use pile::Pile;
use std::collections::BTreeMap;
use std::collections::HashMap;

fn visit(
    deck: Deck,
    pilemap: &mut HashMap<u64, Pile>,
    visited: &mut BTreeMap<u64, u32>,
    orig_min_chaos: u32,
    level: u32,
) -> u32 {
    if level == 10 {
        return orig_min_chaos;
    }
    let mut min_chaos = orig_min_chaos;
    let hash = deck.hash();
    if let Some(chaos) = visited.get(&hash) {
        if *chaos < min_chaos {
            return *chaos;
        }
        return min_chaos;
    }

    let chaos = deck.chaos(&pilemap);
    if chaos < min_chaos {
        min_chaos = chaos;
    }
    //println!("Visit at level {} {}/{}", level, chaos, min_chaos);
    visited.insert(hash, chaos);
    let moves = deck.get_moves(&pilemap, true);
    for m in &moves {
        let newdeck = deck.apply_move(m, pilemap);
        if newdeck.is_won(&pilemap) {
            return 0;
        }
        min_chaos = visit(newdeck, pilemap, visited, min_chaos, level + 1);
    }
    min_chaos
}

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    // need to make this implicit
    let mut pilemap: HashMap<u64, Pile> = HashMap::new();
    let deck = Deck::parse(&contents, &mut pilemap);

    const MAX_CHAOS: u32 = 80000;
    loop {
        let moves = deck.get_moves(&pilemap, false);
        let mut bestchaos = MAX_CHAOS;
        // invalid move
        let mut bestmove = Move::off(11, 0);
        for m in &moves {
            let mut visited = BTreeMap::new();
            deck.explain_move(m, &pilemap);
            let newdeck = deck.apply_move(m, &mut pilemap);

            let newchaos = visit(newdeck, &mut pilemap, &mut visited, MAX_CHAOS, 0);
            println!("Visited in total: {} -> {}", visited.len(), newchaos);
            if newchaos < bestchaos {
                bestchaos = newchaos;
                bestmove = *m;
            }
        }
        break;
    }
}
