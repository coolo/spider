use crate::pile::Pile;
use fasthash::{farm::Hasher64, FastHasher};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hasher;

#[derive(Debug, Copy, Clone)]
pub struct Deck {
    play: [u64; 10],
    talon: [u64; 5],
    off: u64,
}

#[derive(Debug, Copy, Clone)]
pub struct Move {
    off: bool,
    talon: bool,
    from: u8,
    to: u8,
    index: u8,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(index: {}, from: {}, to: {})",
            self.index, self.from, self.to
        )
    }
}

impl Move {
    pub fn regular(from: usize, to: usize, index: usize) -> Move {
        Move {
            talon: false,
            off: false,
            from: from as u8,
            to: to as u8,
            index: index as u8,
        }
    }
    pub fn from_talon(from: usize) -> Move {
        Move {
            talon: true,
            off: false,
            from: from as u8,
            to: 0,
            index: 0,
        }
    }
    pub fn off(from: usize, index: usize) -> Move {
        Move {
            talon: false,
            off: true,
            from: from as u8,
            to: 0,
            index: index as u8,
        }
    }

    pub fn from(&self) -> usize {
        self.from as usize
    }

    pub fn to(&self) -> usize {
        self.to as usize
    }

    pub fn index(&self) -> usize {
        self.index as usize
    }
}
impl Deck {
    pub fn hash(&self) -> u64 {
        let mut h = Hasher64::new();
        for i in 0..10 {
            h.write_u64(self.play[i]);
        }
        for i in 0..5 {
            h.write_u64(self.talon[i])
        }
        h.write_u64(self.off);
        h.finish()
    }

    pub fn is_won(&self, pilemap: &HashMap<u64, Pile>) -> bool {
        let off = pilemap.get(&self.off).expect("valid pile");
        off.count() == 8
    }

    pub fn parse(contents: &String, pilemap: &mut HashMap<u64, Pile>) -> Deck {
        let mut newdeck = Deck {
            play: [0; 10],
            talon: [0; 5],
            off: 0,
        };
        let mut index = 0;
        for line in contents.lines() {
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
                    let parsed = Pile::parse(pile, pilemap);
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
        newdeck
    }

    pub fn to_string(&self, pilemap: &HashMap<u64, Pile>) -> String {
        let mut result = String::new();
        for i in 0..10 {
            result += &format!("Play{}: {}\n", i, pilemap[&self.play[i]].to_string());
        }
        for i in 0..5 {
            result += &format!("Deck{}: {}\n", i, pilemap[&self.talon[i]].to_string());
        }
        result += &format!("Off: {}", pilemap[&self.off].to_string());
        result
    }

    pub fn get_moves(&self, pilemap: &HashMap<u64, Pile>) -> Vec<Move> {
        let mut vec = Vec::new();

        let mut next_talon: Option<usize> = None;
        for i in 0..5 {
            let talon = &pilemap[&self.talon[i]];
            if !talon.is_empty() {
                next_talon = Some(i);
                break;
            }
        }
        // an optimization - only move to first empty
        let mut one_is_empty = false;

        // translate ID into reference for quicker access
        let mut play_refs = Vec::new();
        for i in 0..10 {
            play_refs.push(&pilemap[&self.play[i]]);
        }

        for from in 0..10 {
            if play_refs[from].is_empty() {
                one_is_empty = true;
                continue;
            }
            let from_pile = play_refs[from];
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

                if index > 1 {
                    let next_card = from_pile.at(index - 1);
                    if next_card.suit() == top_suit && next_card.rank() == top_rank + 1 {
                        //println!("Skip {} {}", current.to_string(), next_card.to_string());
                        index -= 1;
                        continue;
                    }
                }
                let mut moved_to_empty = false;

                for to in 0..10 {
                    if to == from {
                        continue;
                    }
                    let to_pile = play_refs[to];
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
                        moved_to_empty = true;
                    }
                    vec.push(Move::regular(from, to, index));
                }

                if index == 0 {
                    break;
                };
                index -= 1;
            }
        }
        match self.prune_moves(&mut vec, &play_refs) {
            None => {
                if !one_is_empty && next_talon.is_some() {
                    vec.push(Move::from_talon(next_talon.unwrap()));
                }
                vec
            }
            Some(m) => {
                //println!("Pruning");
                vec.retain(|&x| {
                    x.from == m.from && x.to == m.to && !x.off && x.index == m.index && !x.talon
                });
                vec
            }
        }
    }

    fn prune_moves(&self, moves: &Vec<Move>, play_refs: &Vec<&Pile>) -> Option<Move> {
        for m in moves {
            if m.off || m.talon {
                continue;
            }
            let to_pile = play_refs[m.to()];
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

    pub fn explain_move(&self, m: &Move, pilemap: &HashMap<u64, Pile>) -> () {
        if m.talon {
            println!("Draw another talon");
            return;
        }
        if m.off {
            println!("Move a sequence from {} to the off", m.from + 1);
            return;
        }
        // happy casting to avoid storing every index as 64 bits
        let from_pile = &pilemap[&self.play[m.from()]];
        let to_pile = &pilemap[&self.play[m.to()]];
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
            m.from + 1,
            m.to + 1,
            from_card,
            to_card
        );
    }

    pub fn apply_move(&self, m: &Move, mut pilemap: &mut HashMap<u64, Pile>) -> Deck {
        let mut newdeck = self.clone();

        if m.talon {
            let from_pile = m.from();
            for to in 0..10 {
                let mut c = pilemap
                    .get(&self.talon[from_pile])
                    .expect("valid pile")
                    .at(to);
                c.set_faceup(true);
                newdeck.play[to] = Pile::add_card(self.play[to], c, &mut pilemap);
            }
            newdeck.talon[m.from()] = Pile::parse("", &mut pilemap).unwrap();
            return newdeck;
        }

        if m.off {
            let from_index = m.from();
            let from_pile = pilemap.get(&self.play[from_index]).expect("valid pile");
            let c = from_pile.at(from_pile.count() - 13);
            newdeck.off = Pile::add_card(self.off, c, &mut pilemap);
            newdeck.play[m.from()] =
                Pile::remove_cards(self.play[m.from()], m.index(), &mut pilemap);
            return newdeck;
        }
        newdeck.play[m.to()] = Pile::copy_from(
            self.play[m.to()],
            self.play[m.from()],
            m.index(),
            &mut pilemap,
        );
        newdeck.play[m.from()] = Pile::remove_cards(self.play[m.from()], m.index(), &mut pilemap);
        newdeck
    }
}

#[cfg(test)]
mod decktests {
    use super::*;

    #[test]
    fn parse() {
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
Deck0: 
Deck1: 
Deck2: 
Deck3: 
Deck4: 
Off: KS KH";
        let mut hashmap = HashMap::new();
        let deck = Deck::parse(&text.to_string(), &mut hashmap);
        assert_eq!(deck.to_string(&hashmap), text);
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
Deck0: 
Deck1: 
Deck2: 
Deck3: 
Deck4: 
Off: KS KH";
        let mut hashmap = HashMap::new();
        let deck = Deck::parse(&text.to_string(), &mut hashmap);
        let moves = deck.get_moves(&hashmap);
        // pick 2H+AH to move to 3H
        assert_eq!(moves.len(), 1);
        let m = moves[0];
        assert_eq!(m.from, 2);
        assert_eq!(m.to, 3);
        assert_eq!(m.index, 7);
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
Deck0: 
Deck1: 
Deck2: 
Deck3: 
Deck4: 
Off: KS KH";
        let mut hashmap = HashMap::new();
        let deck = Deck::parse(&text.to_string(), &mut hashmap);
        let moves = deck.get_moves(&hashmap);
        for m in &moves {
            deck.explain_move(m, &hashmap);
        }
        // pick 5H to move to 6H
        assert_eq!(moves.len(), 1);
        let m = moves[0];
        assert_eq!(m.from, 9);
        assert_eq!(m.to, 3);
        assert_eq!(m.index, 0);
    }
}
