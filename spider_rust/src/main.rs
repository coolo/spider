use std::fs;
mod card;
mod deck;
mod moves;
mod pile;
use deck::Deck;
use moves::Move;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::BinaryHeap;

#[derive(Copy, Clone, Eq, PartialEq)]
struct WeightedMove {
    m: Move,
    weight: i32,
}

impl Ord for WeightedMove {
    fn cmp(&self, other: &Self) -> Ordering {
        other.weight.cmp(&self.weight)
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for WeightedMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

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

    let mut chaos = deck.playable() as i32;
    if chaos == 104 {
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

fn play(deck: Deck, path: &mut BTreeSet<u64>, move_count: usize) -> bool {
    path.insert(deck.hash(0));
    let moves = deck.get_moves(false);
    let mut ordered = BinaryHeap::new();
    const MAX_CHAOS: i32 = 80000;
    if move_count > 30 {
        return false;
    }
    for m in &moves {
        let mut visited = BTreeMap::new();
        //deck.explain_move(m);
        let newdeck = deck.apply_move(m);
        if path.contains(&newdeck.hash(0)) {
            continue;
        }
        let newchaos = visit(newdeck, &mut visited, MAX_CHAOS, 5);
        //println!("Visited in total: {} -> {}", visited.len(), newchaos);
        ordered.push(WeightedMove {
            m: *m,
            weight: newchaos,
        });
    }
    if ordered.is_empty() {
        println!("No moves found {}", deck.chaos());
        if deck.chaos() == 0 {
            println!("WON!");
            return true;
        }
        return false;
    }

    let mut _o = 0;
    for bestmove in ordered {
        _o += 1;
        //deck.explain_move(&bestmove.m);
        let new_deck = deck.apply_move(&bestmove.m);
        let mut mc = move_count;
        if !bestmove.m.is_off() {
            mc += 1;
        }
        //println!("{}/{} {}", move_count, o, deck.chaos());
        //deck.to_string()
        if play(new_deck, path, mc) {
            deck.explain_move(&bestmove.m);
            return true;
        }
    }
    false
}

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let deck = Deck::parse(&contents);
    let mut path: BTreeSet<u64> = BTreeSet::new();

    println!("{}", deck.shortest_path(50000000).expect("win"));
}
