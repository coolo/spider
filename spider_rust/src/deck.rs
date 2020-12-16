use crate::card::Card;
use crate::moves::Move;
use crate::pile::Pile;
use fasthash::farm;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::collections::BinaryHeap;
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::Arc;

const MAX_MOVES: usize = 200;

#[derive(Debug, Clone)]
pub struct Deck {
    play: [u32; 10],
    talon: [u32; 5],
    off: u32,
    moves: [Move; MAX_MOVES],
    moves_index: usize,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct WeightedMove {
    deck: usize,
    chaos: u32,
    playable: u32,
    hash: u64,
}

impl Ord for WeightedMove {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .chaos
            .cmp(&self.chaos)
            .then(self.playable.cmp(&other.playable))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for WeightedMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Deck {
    pub fn hash(&self, seed: u32) -> u64 {
        let plays = self.play.as_ptr() as *const u8;
        let talons = self.talon.as_ptr() as *const u8;
        unsafe {
            let mut bytes: [u8; 68] = MaybeUninit::zeroed().assume_init();
            ptr::copy_nonoverlapping(plays, bytes.as_mut_ptr(), 40);
            ptr::copy_nonoverlapping(talons, bytes.as_mut_ptr().offset(40), 20);
            let t = std::mem::transmute::<u32, [u8; 4]>(self.off);
            ptr::copy_nonoverlapping(t.as_ptr(), bytes.as_mut_ptr().offset(60), 4);
            let t = std::mem::transmute::<u32, [u8; 4]>(seed);
            ptr::copy_nonoverlapping(t.as_ptr(), bytes.as_mut_ptr().offset(64), 4);
            farm::hash64(bytes)
        }
    }

    #[allow(dead_code)]
    pub fn is_won(&self) -> bool {
        Pile::get(self.off).count() == 8
    }

    pub fn parse(contents: &String) -> Deck {
        let mut newdeck = Deck {
            play: [0; 10],
            talon: [0; 5],
            off: 0,
            moves_index: 0,
            moves: [Move::invalid(); MAX_MOVES],
        };
        // that should be enough :)
        let mut index = 0;
        for line in contents.lines() {
            if line.starts_with("#") {
                continue;
            }
            let mut two = line.split(":");
            match two.next() {
                None => {
                    break;
                }
                Some(_) => {}
            }
            match two.next() {
                None => {
                    break;
                }
                Some(pile) => {
                    let parsed = Pile::parse(pile);
                    match parsed {
                        None => panic!("Failed to parse {}", pile),
                        Some(pile) => match index {
                            0..=9 => newdeck.play[index] = pile,
                            10..=14 => newdeck.talon[index - 10] = pile,
                            15 => newdeck.off = pile,
                            _ => panic!("We went too far"),
                        },
                    }
                }
            }
            index += 1;
        }
        assert_eq!(index, 16);
        newdeck
    }

    pub fn win_moves(&self) -> Vec<Move> {
        //Vec::new(self.moves)
        let mut ret = vec![];
        ret.extend(self.moves.iter());
        ret.truncate(self.moves_index);
        ret
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for i in 0..10 {
            result += &format!("Play{}: {}\n", i, Pile::get(self.play[i]).to_string());
        }
        for i in 0..5 {
            result += &format!("Deal{}: {}\n", i, Pile::get(self.talon[i]).to_string());
        }
        result += &format!("Off: {}", Pile::get(self.off).to_string());
        result
    }

    pub fn get_moves(&self, prune: bool) -> Vec<Move> {
        let mut vec = Vec::new();

        let mut next_talon: Option<usize> = None;
        for i in 0..5 {
            let talon = Pile::get(self.talon[i]);
            if !talon.is_empty() {
                next_talon = Some(i);
                break;
            }
        }
        // can't pull the talon if it turns true
        let mut one_is_empty = false;

        // translate ID into reference for quicker access
        let mut play_refs = Vec::new();
        for i in 0..10 {
            play_refs.push(Pile::get(self.play[i]));
        }

        for from in 0..10 {
            if play_refs[from].is_empty() {
                one_is_empty = true;
                continue;
            }
            let from_pile = &play_refs[from];
            let mut index = from_pile.count() - 1;
            let top_card = from_pile.at(index);

            let top_suit = top_card.suit();
            let mut top_rank = top_card.rank() - 1;

            loop {
                let current = from_pile.at(index);
                if !current.faceup() {
                    break;
                }
                if current.suit() != top_suit {
                    break;
                }
                if top_rank + 1 != current.rank() {
                    break;
                }
                top_rank = current.rank();

                if from_pile.count() - index == 13 {
                    // off move
                    vec.clear();
                    vec.push(Move::off(from, index));
                    //println!("Found off move");
                    return vec;
                }

                let mut broken_sequence = false;
                if index > 0 {
                    let next_card = from_pile.at(index - 1);
                    if next_card.faceup()
                        && next_card.suit() == top_suit
                        && next_card.rank() == top_rank + 1
                    {
                        broken_sequence = true;
                        if prune {
                            //println!("Skip {} {}", current.to_string(), next_card.to_string());
                            index -= 1;
                            continue;
                        }
                    }
                }

                let mut moved_to_empty = false;

                for to in 0..10 {
                    if to == from {
                        continue;
                    }
                    let to_pile = &play_refs[to];
                    let to_count = to_pile.count();

                    if to_count > 0 {
                        let top_card = to_pile.at(to_count - 1);
                        if top_card.rank() != top_rank + 1 {
                            continue;
                        }
                    } else if moved_to_empty {
                        // if there is a talon left to draw the empty cell
                        // we move to does matter. In the endgame not at all
                        if next_talon.is_none() {
                            continue;
                        }
                    } else {
                        // while talons are there, optimisations are evil
                        // but in end game we have more options
                        if next_talon.is_none() {
                            if index == 0 {
                                // forbid moves between empty cells once the talons are gone
                                continue;
                            }
                            // there is no plausible reason to split up sequences in end game
                            if broken_sequence {
                                continue;
                            }
                        }
                        moved_to_empty = true;
                    }
                    vec.push(Move::regular(from, to, index));
                }

                if index == 0 {
                    break;
                }
                index -= 1;
            }
        }
        if !prune {
            if !one_is_empty && next_talon.is_some() {
                vec.push(Move::from_talon(next_talon.unwrap()));
            }
            return vec;
        }
        match self.prune_moves(&mut vec, &play_refs) {
            None => {
                if !one_is_empty && next_talon.is_some() {
                    vec.push(Move::from_talon(next_talon.unwrap()));
                }
                vec
            }
            Some(m) => {
                vec.retain(|&x| x == m);
                vec
            }
        }
    }

    fn prune_moves(&self, moves: &Vec<Move>, play_refs: &Vec<Arc<Pile>>) -> Option<Move> {
        for m in moves {
            assert!(!m.is_off() && !m.is_talon());
            let to_pile = &play_refs[m.to()];
            if to_pile.count() == 0 {
                continue;
            }
            let from_suit = play_refs[m.from()].at(m.index()).suit();
            let to_suit = to_pile.at(to_pile.count() - 1).suit();
            if to_suit == from_suit {
                let newm: Move = *m;
                return Some(newm.clone());
            }
        }
        None
    }

    pub fn explain_move(&self, m: &Move) -> () {
        if m.is_talon() {
            println!("Draw another talon");
            return;
        }
        if m.is_off() {
            println!("Move a sequence from {} to the off", m.from() + 1);
            return;
        }
        // happy casting to avoid storing every index as 64 bits
        let from_pile = Pile::get(self.play[m.from()]);
        let to_pile = Pile::get(self.play[m.to()]);
        let from_card = from_pile.at(m.index()).to_string();
        let mut to_card = String::from("Empty");
        if to_pile.count() > 0 {
            let c = to_pile.at(to_pile.count() - 1);
            to_card = c.to_string();
        }
        let mut count = from_pile.count();
        count -= m.index();
        println!(
            "Move {} cards from {} to {} - {}->{}",
            count,
            m.from() + 1,
            m.to() + 1,
            from_card,
            to_card
        );
    }

    pub fn chaos(&self) -> u32 {
        let mut result = 0;
        for i in 0..10 {
            result += Pile::get(self.play[i]).chaos();
        }
        for i in 0..5 {
            if !Pile::get(self.talon[i]).is_empty() {
                result += 40;
            }
        }
        result
    }

    pub fn playable(&self) -> u32 {
        let mut result: u32 = 0;
        for i in 0..10 {
            result += Pile::get(self.play[i]).playable();
        }
        result + 13 * (Pile::get(self.off).count() as u32)
    }

    pub fn apply_move(&self, m: &Move) -> Deck {
        let mut newdeck = self.clone();
        newdeck.moves[newdeck.moves_index] = *m;
        newdeck.moves_index += 1;
        if newdeck.moves_index >= MAX_MOVES {
            panic!("Way too deep");
        }

        if m.is_talon() {
            let from_pile = m.from();
            for to in 0..10 {
                let mut c = Pile::get(self.talon[from_pile]).at(to);
                c.set_faceup(true);
                newdeck.play[to] = Pile::add_card(self.play[to], c);
            }
            newdeck.talon[m.from()] = Pile::parse("").unwrap();
            return newdeck;
        }

        if m.is_off() {
            let from_index = m.from();
            let from_pile = Pile::get(self.play[from_index]);
            let c = from_pile.at(from_pile.count() - 13);
            newdeck.off = Pile::add_card(self.off, c);
            newdeck.play[m.from()] = Pile::remove_cards(self.play[m.from()], m.index());
            return newdeck;
        }
        newdeck.play[m.to()] = Pile::copy_from(self.play[m.to()], self.play[m.from()], m.index());
        newdeck.play[m.from()] = Pile::remove_cards(self.play[m.from()], m.index());
        newdeck
    }

    fn pick_one_for_shortest_path(
        deck: &Deck,
        visited: &mut BTreeSet<u64>,
        new_unvisited: &mut Vec<Deck>,
        new_unvisited_tosort: &mut Vec<WeightedMove>,
    ) -> Option<u64> {
        //let _output = visited.len() % 100000 == 0;
        let output = false;
        if output {
            println!("{} {} {}", deck.to_string(), deck.playable(), deck.chaos());
        }

        let moves = deck.get_moves(false);
        let mut candidates = BinaryHeap::new();
        let playable = deck.playable();
        let chaos = deck.chaos();
        // we have one sorted and one unsorted to avoid the sorting
        // copying decks
        let mut newdecks = vec![];
        for m in &moves {
            if output {
                deck.explain_move(m);
            }
            let newdeck = deck.apply_move(m);
            let hash = newdeck.hash(0);
            newdecks.push(newdeck.clone());

            let newplayable = newdeck.playable();
            let newchaos = newdeck.chaos();
            if output {
                println!(
                    "PLAY {} -> {} CHAOS {} -> {}",
                    playable, newplayable, chaos, newchaos
                );
            }
            candidates.push(WeightedMove {
                chaos: newchaos,
                playable: newplayable,
                hash: hash,
                deck: newdecks.len() - 1,
            });
        }
        let mut onegood = false;

        for candidate in candidates {
            if candidate.chaos < chaos || candidate.playable > playable {
                onegood = true;
            } else if onegood {
                break;
            }
            if !visited.contains(&candidate.hash) {
                if output {
                    println!("Candidate {} {}", candidate.chaos, candidate.playable);
                }
                {
                    visited.insert(candidate.hash);
                    new_unvisited.push(newdecks[candidate.deck].clone());
                    if newdecks[candidate.deck].is_won() {
                        return Some(candidate.hash);
                    }
                    let mut nc = candidate;
                    nc.deck = new_unvisited.len() - 1;
                    new_unvisited_tosort.push(nc);
                }
            }
        }
        None
    }

    pub fn full_deck(n_suits: usize) -> Vec<Card> {
        let mut cards = vec![];
        for suit in 0..4 {
            for rank in 1..=13 {
                let mut nsuit = suit;
                if n_suits == 1 {
                    nsuit = 0;
                }
                if n_suits == 2 {
                    nsuit = suit % 2;
                }
                let c = Card::known(nsuit, rank);
                cards.push(Card::new(c.value()));
                cards.push(c);
            }
        }
        cards
    }

    pub fn shuffle_unknowns(&mut self, n_suits: usize) {
        let mut cards = Deck::full_deck(n_suits);
        for i in 0..10 {
            Pile::get(self.play[i]).remove_known(&mut cards);
        }
        for i in 0..5 {
            Pile::get(self.talon[i]).remove_known(&mut cards);
        }
        let off = Pile::get(self.off);
        for i in 0..off.count() {
            let suit = off.at(i).suit();
            for rank in 1..=13 {
                let c = Card::known(suit, rank);
                let index = cards
                    .iter()
                    .position(|x| x.is_same_card(&c))
                    .expect("card is in");
                cards.remove(index);
            }
        }
        let mut rng = thread_rng();
        cards.shuffle(&mut rng);
        println!("Cards {}", Card::vec_as_string(&cards));
        for i in 0..10 {
            self.play[i] = Pile::get(self.play[i]).pick_unknown(&mut cards);
        }
        for i in 0..5 {
            self.talon[i] = Pile::get(self.talon[i]).pick_unknown(&mut cards);
        }
        if cards.len() > 0 {
            panic!("There are cards left: {}", Card::vec_as_string(&cards));
        }
    }

    pub fn shortest_path(&mut self, cap: usize, limit: usize) -> Option<i32> {
        let mut unvisted: Vec<Deck> = Vec::new();
        unvisted.push(self.clone());
        // just append
        let mut new_unvisited: Vec<Deck> = Vec::new();
        // sort only the index
        let mut new_unvisited_tosort: Vec<WeightedMove> = Vec::new();
        let mut visited = BTreeSet::new();
        visited.insert(self.hash(0));

        let mut depth: i32 = 0;

        loop {
            if visited.len() > limit {
                return None;
            }
            for deck in &unvisted {
                let deck = deck.clone();
                if let Some(res) = Deck::pick_one_for_shortest_path(
                    &deck,
                    &mut visited,
                    &mut new_unvisited,
                    &mut new_unvisited_tosort,
                ) {
                    println!("WON! {} {}", depth + 1, visited.len());
                    let mut iter = new_unvisited.iter();
                    loop {
                        match iter.next() {
                            None => break,
                            Some(val) => {
                                if res == val.hash(0) {
                                    self.moves = val.moves.clone();
                                    return Some(depth + 1);
                                }
                            }
                        }
                    }
                    assert!(false);
                }
            }

            unvisted.clear();
            new_unvisited_tosort.sort_unstable();
            new_unvisited_tosort.reverse();

            let mut iterator = new_unvisited_tosort.iter();
            let mut printed = false;
            for _ in 0..cap {
                if let Some(wm) = iterator.next() {
                    if !printed {
                        println!(
                            "{}/{} {} {}",
                            depth,
                            new_unvisited.len(),
                            wm.chaos,
                            wm.playable
                        );
                        //println!("{}", new_unvisited[wm.deck].to_string());
                        printed = true;
                    }
                    unvisted.push(new_unvisited[wm.deck].clone());
                } else {
                    break;
                }
            }

            new_unvisited_tosort.clear();
            new_unvisited.clear();

            depth += 1;
            if unvisted.len() == 0 {
                break;
            }
        }

        Some(-1 * depth)
    }

    pub fn top_card_unknown(&self, index: usize) -> bool {
        let pile = Pile::get(self.play[index]);
        if pile.count() == 0 {
            return false;
        }
        pile.at(pile.count() - 1).is_unknown()
    }

    pub fn replace_play_card(&mut self, play: usize, index: usize, c: &Card) {
        let mut c = Card::new(c.value());
        c.set_faceup(Pile::get(self.play[play]).at(index).faceup());
        self.play[play] = Pile::replace_at(self.play[play], index, &c);
    }
}

#[cfg(test)]
mod decktests {
    use super::*;

    #[test]
    fn parse() {
        let text = "Play0: KS QS JS TS 9S 8S 7S 6S
Play1: |AH |4H QH JH TH 9H 8H 7H 6H 5H
Play2: |TH |2S |JS |KS |KS QH JH 2H AH
Play3: |6H 3H
Play4: |TH |3S |TS 9S 8S KH QH JH TS 9H 8H 7H 6H 5S 4S 3S 2S AS
Play5: |9S |9H 8H 7H
Play6: |7S |QS |KH |4H 3H 2H
Play7: |8S |JS |7S AS 5H 4H 2S AS KH QS 6S 5S 4S 3S
Play8: 6S 5S 4S 3H 2H AH
Play9: 5H
Deal0: 
Deal1: 
Deal2: 
Deal3: 
Deal4: 
Off: KS KH";
        let deck = Deck::parse(&text.to_string());
        assert_eq!(deck.to_string(), text);
    }

    #[test]
    fn taketwo() {
        let text = "Play0: KS QS JS TS 9S 8S 7S AS
Play1: |AH |4H QH JH TH 9H 8H 7H 6H 5H
Play2: |TH |2S |JS |KS |KS QH JH 2H AH
Play3: |6H 3H
Play4: |TH |3S |TS 9S 8S KH QH JH TS 9H 8H 7H 6H 5S 4S 3S 2S AS
Play5: |9S |9H 8H 7H
Play6: |7S |QS |KH |4H 3H 2H
Play7: |8S |JS |7S 6S 5H 4H 2S AS KH QS 6S 5S 4S 3S
Play8: 6S 5S 4S 3H 2H AH
Play9: 5H
Deal0: 
Deal1: 
Deal2: 
Deal3: 
Deal4: 
Off: KS KH";
        let deck = Deck::parse(&text.to_string());
        let moves = deck.get_moves(true);
        // pick 2H+AH to move to 3H
        assert_eq!(moves.len(), 1);
        let m = moves[0];
        assert_eq!(m.from(), 2);
        assert_eq!(m.to(), 3);
        assert_eq!(m.index(), 7);
    }

    #[test]
    fn pickone() {
        let text = "Play0: KS QS JS TS 9S 8S 7S AS
Play1: |AH |4H QH JH TH 9H 8H 7H 6H 5H
Play2: |TH |2S |JS |KS |KS QH JH 2H AH
Play3: |3H 6H
Play4: |TH |3S |TS 9S 8S KH QH JH TS 9H 8H 7H 6H 5S 4S 3S 2S AS
Play5: |7H |9H 8H 9S
Play6: |7S |QS |KH |4H 3H 2H
Play7: |8S |JS |7S 6S 5H 4H 2S AS KH QS 6S 5S 4S 3S
Play8: 6S 5S 4S 3H 2H AH
Play9: 5H
Deal0: 
Deal1: 
Deal2: 
Deal3: 
Deal4: 
Off: KS KH";
        let deck = Deck::parse(&text.to_string());
        let moves = deck.get_moves(true);
        for m in &moves {
            deck.explain_move(m);
        }
        // pick 5H to move to 6H
        assert_eq!(moves.len(), 1);
        let m = moves[0];
        assert_eq!(m.from(), 9);
        assert_eq!(m.to(), 3);
        assert_eq!(m.index(), 0);
    }

    #[test]
    fn dont_move_between_empty() {
        let text = "Play0: 7H 6H 5H AS
        Play1: 
        Play2: KS
        Play3: TH
        Play4: |3S |9S |TS TH
        Play5: |9S |9H 8H 4S
        Play6: |7S |QS |KH |4H 3H 2S QH JH KH QH JS QS KS
        Play7: |8S |JS |6S 7S
        Play8: |6S |8S |AH |5S 4H 3H 2H AH
        Play9: 5H 2H JH TS 9H 8H 7H 6H 5S 4S 3S 2S AS
        Deal0: 
        Deal1: 
        Deal2: 
        Deal3: 
        Deal4: 
        Off: KS KH KH KS";
        let deck = Deck::parse(&text.to_string());
        let moves = deck.get_moves(true);
        for m in &moves {
            deck.explain_move(&m);
            // all moves are to empty
            assert_eq!(m.to(), 1);
            // moves from empty to empty are forbidden
            // if the talons are done
            assert!(m.index() > 0);
        }
    }

    #[test]
    fn pick_good_ones() {
        let text = "Play0: AS
Play1: KS QS
Play2: 2H AH
Play3: 
Play4: |TH |3S |TS 9S 8S
Play5: |9S |9H 8H 7H 6H 5S 4S 3S 2S AS
Play6: |7S |QS |KH |4H 3H 2S QH JH KH QH JS TH 9H 8H 7H 6H 5H 4H 3H 2H AH
Play7: |8S |JS |7S 6S 5S 4S
Play8: 6S 5H
Play9: KS JH TS
Deal0: 
Deal1: 
Deal2: 
Deal3: 
Deal4: 
Off: KS KH KH KS";
        let deck = Deck::parse(&text.to_string());
        let moves = deck.get_moves(true);
        // pick 9S to move to TS to uncover the other TS
        assert_eq!(moves.len(), 1);
        let m = moves[0];
        assert_eq!(m.from(), 4);
        assert_eq!(m.to(), 9);
        assert_eq!(m.index(), 3);
    }

    #[test]
    fn pick_empty() {
        let text = "Play0: AS
Play1: KS QS
Play2: 2H AH
Play3: 
Play4: |TH |3S |TS 9S 8S
Play5: |9S |9H 8H 7H 6H 5S 4S 3S 2S AS
Play6: |7S |QS |KH |4H 3H 2S QH JH KH QH JS TH 9H 8H 7H 6H 5H 4H 3H 2H AH
Play7: |8S |JS |7S 6S 5S 4S
Play8: 6S 5H
Play9: TS JH KS
Deal0: 
Deal1: 
Deal2: 
Deal3: 
Deal4: 
Off: KS KH KH KS";
        let deck = Deck::parse(&text.to_string());
        let moves = deck.get_moves(true);
        for m in &moves {
            deck.explain_move(m);
            // all moves are to empty
            assert_eq!(m.to(), 3);
            // the first two piles should not move
            assert!(m.from() > 3);
        }
    }

    #[test]
    fn chaos0() {
        let text = "Play0: 
Play1: 
Play2: 
Play3: 
Play4: 
Play5: 
Play6: 
Play7: 
Play8: 
Play9: 
Deal0: 
Deal1: 
Deal2: 
Deal3: 
Deal4: 
Off: KS KS KS KS KH KH KH KH";
        let deck = Deck::parse(&text.to_string());
        assert_eq!(deck.chaos(), 0);
        assert_eq!(deck.playable(), 104);
    }

    #[test]
    fn chaos2() {
        let text = "Play0: KH QH JH TH 
Play1: 9H 
Play2: 8H 7H 6H 5H 4H 3H 2H AH
Play3: 
Play4: 
Play5: 
Play6: 
Play7: 
Play8: 
Play9: 
Deal0: 
Deal1: 
Deal2: 
Deal3: 
Deal4: 
Off: KS KS KS KS KH KH KH";
        let deck = Deck::parse(&text.to_string());
        assert_eq!(deck.chaos(), 16);
        assert_eq!(deck.playable(), 104);
    }

    #[test]
    fn shortest_path1() {
        let text = "Play0: KH QH JH TH 
        Play1: 9H 
        Play2: 8H 7H 6H 5H 4H 3H 2H AH
        Play3: 
        Play4: 
        Play5: 
        Play6: 
        Play7: 
        Play8: 
        Play9: 
        Deal0: 
        Deal1: 
        Deal2: 
        Deal3: 
        Deal4: 
        Off: KS KS KS KS KH KH KH";
        let mut deck = Deck::parse(&text.to_string());
        assert_eq!(deck.shortest_path(3400, 100).expect("winnable"), 3);
    }

    #[test]
    fn shortest_path2() {
        // win in 28 moves
        // 7->0 8->7 8->3 1->3 8->3 9->3 9->8 9->4
        // 0->9 9->6 6->off 3->4 8->3 6->5 6->4 9->6 7->6 9->3
        // 6->7 7->6 6->3 3->5 5->6 6->off 6->2 4->2 6->2 2->off
        let text = "Play0: TH 9H 8H 7H 6H 5H 4H 3H
        Play1: 7S
        Play2: KS
        Play3: TH 9S
        Play4: JS
        Play5: 
        Play6: |AS |QS |KH |4H 3H 2S QH JH KH QH
        Play7: 2H AH
        Play8: |6S |8S AH
        Play9: 5H 2H JH TS 9H 8H 7H 6H 5S 4S 3S
        Deal0: 
        Deal1: 
        Deal2: 
        Deal3: 
        Deal4: 
        Off: KS KH KH KS KS";
        let mut deck = Deck::parse(&text.to_string());
        let res = deck.shortest_path(3400, 5000);
        //assert_eq!(res.expect("winnable"), 28);
        assert!(res.is_none()); // requires a little more capacity
    }

    #[test]
    fn shortest_path3() {
        // win in 17: 4->8 6->4 6->5 4->5 2->5 5->4 5->1 4->1 1->6 6->off 7->6 6->3 5->3 7->3 6->3 8->3 3->off
        let text = "Play0:
        Play1: QH JH TH
        Play2: 2H AH
        Play3: KS
        Play4: 5S 4S 3S 2S AS
        Play5: |9S |9H 8H 7H 6H 5H
        Play6: |7S |QS |KH |4H 3H
        Play7: |8S JS TS
        Play8: 6S
        Play9:
        Deal0:
        Deal1:
        Deal2:
        Deal3:
        Deal4:
        Off: KS KH KH KS KH KS";
        let mut deck = Deck::parse(&text.to_string());
        // win in 17 moves
        let res = deck.shortest_path(5400, 80000);
        assert_eq!(res.expect("winnable"), 17);
    }

    #[test]
    fn shortest_path4() {
        let text = "Play0: JS TS 9S 8S 7S 6S 5S 4S AS TH 9H 8H 7H 6H 5H 4H 3H 2H AH
        Play1: KH QH JH 8H 6H 5H 4H 8H 2S AS
        Play2: JH TS 9S 8S
        Play3: KH TH 3S 2S KH QH
        Play4: |8H |AH 7S 6S 5S 4S 3H 7H 6H 5H 4H KS QS 6H 5S 4S
        Play5: 7H 7S TH 9H
        Play6: |JH AH JS 9H KS QS 5H 8S
        Play7: |7H |9S 6S JH KH 2H 3H 2H
        Play8: TH 9H QS QH KS QH
        Play9: 3S 2S AS JS 4H 3H 2H AH 3S TS
        Deal0: 
        Deal1: 
        Deal2: 
        Deal3: 
        Deal4: 
        Off: KS";
        let mut deck = Deck::parse(&text.to_string());
        // win in 17 moves
        let res = deck.shortest_path(3400, 50000);
        assert_eq!(res.expect("out of options"), -8);
    }

    #[test]
    fn top_card_unknown() {
        let text = "Play0: JS TS 9S 8S 7S 6S 5S 4S AS TH 9H 8H 7H 6H 5H 4H 3H 2H AH
        Play1: KH QH JH 8H 6H 5H 4H 8H 2S AS
        Play2: JH TS 9S 8S
        Play3: KH TH 3S 2S KH QH
        Play4: |8H |AH 7S 6S 5S 4S 3H 7H 6H 5H 4H KS QS 6H 5S 4S
        Play5: 7H 7S TH XX
        Play6: |JH AH JS 9H KS QS 5H 8S
        Play7: |7H |9S 6S JH KH 2H 3H 2H
        Play8: TH 9H QS QH KS QH
        Play9: 3S 2S AS JS 4H 3H 2H AH 3S TS
        Deal0: 
        Deal1: 
        Deal2: 
        Deal3: 
        Deal4: 
        Off: KS";
        let deck = Deck::parse(&text.to_string());
        assert_eq!(deck.top_card_unknown(5), true);
        assert_eq!(deck.top_card_unknown(4), false);
    }
}
