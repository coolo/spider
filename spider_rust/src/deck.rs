use crate::card::Card;
use crate::moves::Move;
use crate::pile::Pile;
use seahash;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

pub const MAX_MOVES: usize = 250;

#[derive(Clone)]
pub struct Deck {
    play: [Rc<Pile>; 10],
    talon: [Rc<Pile>; 5],
    off: Rc<Pile>,
    moves: [Move; MAX_MOVES],
    moves_index: usize,
}

#[derive(Clone)]
struct WeightedMove {
    deck: Rc<Deck>,
    talons: u32,
    hash: u64,
    chaos: u32,
    in_off: u32,
    free_plays: u32,
    playable: u32,
}

impl WeightedMove {
    pub fn from(deck: Rc<Deck>, hash: u64) -> Self {
        Self {
            talons: deck.free_talons(),
            chaos: deck.chaos(),
            hash: hash,
            in_off: deck.in_off(),
            free_plays: deck.free_plays(),
            playable: deck.playable(),
            deck: deck,
        }
    }
}

impl Ord for WeightedMove {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = other.chaos.cmp(&self.chaos);
        if ord != Ordering::Equal {
            return ord;
        }
        let ready1 = self.playable + self.in_off + self.free_plays;
        let ready2 = other.playable + other.in_off + other.free_plays;
        let ord = ready1.cmp(&ready2);
        if ord != Ordering::Equal {
            return ord;
        }
        if self.chaos == 0 {
            // once we are in straight win mode, we go differently
            let ord = self.free_plays.cmp(&other.free_plays);
            if ord != Ordering::Equal {
                return ord;
            }
            // if the number of empty plays is equal, less in the off
            // is actually a benefit (more strongly ordered)
            let ord = other.in_off.cmp(&self.in_off);
            if ord != Ordering::Equal {
                return ord;
            }
        }
        self.hash.cmp(&other.hash)
    }
}

impl PartialEq for WeightedMove {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for WeightedMove {}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for WeightedMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Deck {
    pub fn hash(&self) -> u64 {
        let mut state = seahash::State::new(
            0x16f11fe89b0d677c,
            0xb480a793d8e6c86c,
            0x6fe2e5aaf078ebc9,
            0x14f994a4c5259381,
        );
        for i in 0..10 {
            self.play[i].hash(&mut state)
        }
        for i in 0..5 {
            self.talon[i].hash(&mut state)
        }
        state.finalize()
    }

    #[inline]
    pub fn in_off(&self) -> u32 {
        (self.off.count() as u32) * 13
    }

    #[allow(dead_code)]
    pub fn is_won(&self) -> bool {
        self.off.count() == 8
    }

    pub fn free_plays(&self) -> u32 {
        let mut result = 0;
        for i in 0..10 {
            if self.play[i].count() == 0 {
                result += 1;
            }
        }
        result
    }

    pub fn reset_moves(&mut self) {
        self.moves_index = 0;
    }

    pub fn free_talons(&self) -> u32 {
        // TODO: store as property
        let mut ret = 0;
        for i in 0..5 {
            if self.talon[i].count() == 0 {
                ret += 1
            }
        }
        ret
    }

    pub fn empty() -> Deck {
        Deck {
            play: [
                Rc::clone(&Pile::empty()),
                Rc::clone(&Pile::empty()),
                Rc::clone(&Pile::empty()),
                Rc::clone(&Pile::empty()),
                Rc::clone(&Pile::empty()),
                Rc::clone(&Pile::empty()),
                Rc::clone(&Pile::empty()),
                Rc::clone(&Pile::empty()),
                Rc::clone(&Pile::empty()),
                Rc::clone(&Pile::empty()),
            ],
            talon: [
                Rc::clone(&Pile::empty()),
                Rc::clone(&Pile::empty()),
                Rc::clone(&Pile::empty()),
                Rc::clone(&Pile::empty()),
                Rc::clone(&Pile::empty()),
            ],
            off: Pile::empty(),
            moves_index: 0,
            moves: [Move::invalid(); MAX_MOVES],
        }
    }

    pub fn parse(contents: &String) -> Deck {
        let mut newdeck = Deck::empty();
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

        if index != 16 {
            panic!("Not all piles are parsed");
        }
        newdeck
    }

    pub fn set_play(&mut self, index: usize, pile: Rc<Pile>) {
        self.play[index] = pile;
    }

    pub fn set_talon(&mut self, index: usize, pile: Rc<Pile>) {
        self.talon[index] = pile;
    }

    pub fn set_off(&mut self, pile: Rc<Pile>) {
        self.off = pile;
    }

    pub fn win_moves(&self) -> Vec<Move> {
        let mut ret = vec![];
        ret.extend(self.moves.iter());
        ret.truncate(self.moves_index);
        ret
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for i in 0..10 {
            result += &format!("Play{}: {}\n", i, self.play[i].to_string());
        }
        for i in 0..5 {
            result += &format!("Deal{}: {}\n", i, self.talon[i].to_string());
        }
        result += &format!("Off: {}", self.off.to_string());
        result
    }

    fn next_talon(&self) -> Option<usize> {
        for i in 0..5 {
            if !self.talon[i].is_empty() {
                return Some(i);
            }
        }
        None
    }

    pub fn get_moves(&self, vec: &mut Vec<Move>) {
        vec.clear();

        if self.moves_index == MAX_MOVES - 1 {
            //println!("Too deep {}", self.to_string());
            return;
        }
        let next_talon = self.next_talon();

        // no point in looking
        if next_talon.is_some() && self.playable() < 10 {
            return;
        }
        // can't pull the talon if it turns true
        let mut one_is_empty = false;

        for from in 0..10 {
            if self.play[from].is_empty() {
                one_is_empty = true;
                continue;
            }
            let from_pile = &self.play[from];
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
                    return;
                }

                let mut broken_sequence = 0;
                if index > 0 {
                    let next_card = from_pile.at(index - 1);
                    if current.is_in_sequence_to(&next_card) {
                        broken_sequence = from_pile.count() - index;
                    }
                }

                let mut moved_to_empty = false;

                for to in 0..10 {
                    if to == from {
                        continue;
                    }
                    let to_pile = &self.play[to];
                    let to_count = to_pile.count();

                    if to_count > 0 {
                        let top_card = to_pile.at(to_count - 1);
                        if top_card.rank() != top_rank + 1 {
                            continue;
                        }
                        if broken_sequence > 0 {
                            /*println!(
                                "BS {}->{} {} {} {}",
                                from,
                                to,
                                broken_sequence,
                                to_pile.sequence_of(top_suit),
                                from_pile.sequence_of(top_suit)
                            );*/
                            if to_pile.sequence_of(top_suit) + broken_sequence
                                <= from_pile.sequence_of(top_suit)
                            {
                                continue;
                            }
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
                            if broken_sequence > 0 {
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

        if !one_is_empty && next_talon.is_some() {
            vec.push(Move::from_talon(next_talon.unwrap()));
        }
    }

    pub fn explain_move(&self, m: &Move) -> String {
        if m.is_talon() {
            return format!("Draw another talon");
        }
        if m.is_off() {
            return format!("Move a sequence from {} to the off", m.from() + 1);
        }
        // happy casting to avoid storing every index as 64 bits
        let from_pile = &self.play[m.from()];
        let to_pile = &self.play[m.to()];

        let from_card = from_pile.at(m.index()).to_string();
        let mut to_card = String::from("Empty");
        if to_pile.count() > 0 {
            let c = to_pile.at(to_pile.count() - 1);
            to_card = c.to_string();
        }
        let mut count = from_pile.count();
        count -= m.index();
        if self.result_of_tap(m.from()) == Some(*m) {
            return format!("Tap on {} ({}->{})", m.from() + 1, from_card, m.to() + 1);
        } else {
            return format!(
                "Move {} cards from {} to {} - {}->{}",
                count,
                m.from() + 1,
                m.to() + 1,
                from_card,
                to_card
            );
        }
    }

    pub fn result_of_tap(&self, play: usize) -> Option<Move> {
        let from_pile = &self.play[play];
        let mut index = from_pile.count();
        if index < 1 {
            return None;
        }
        index -= 1;
        let mut top_card = from_pile.at(index);
        while index > 0 && top_card.is_in_sequence_to(&from_pile.at(index - 1)) {
            index -= 1;
            top_card = from_pile.at(index);
        }
        //println!("tap on {} gives {}", play, top_card.to_string());
        let mut candidates = vec![];
        for i in 0..10 {
            if i == play {
                continue;
            }
            let to_pile = &self.play[i];
            if to_pile.count() == 0 || top_card.fits_on_top(&to_pile.at(to_pile.count() - 1)) {
                candidates.push((i, to_pile.sequence_of(top_card.suit())));
            }
        }
        if candidates.len() == 0 {
            return None;
        }
        if candidates.len() == 1 {
            return Some(Move::regular(play, candidates[0].0, index));
        }
        let mut best_sequence: usize = 0;
        for (_, sequence) in candidates.iter() {
            if *sequence > best_sequence {
                best_sequence = *sequence;
            }
        }
        candidates.retain(|&x| x.1 == best_sequence);
        if candidates.len() == 1 {
            return Some(Move::regular(play, candidates[0].0, index));
        }
        return None;
    }

    pub fn pile_chaos(&self) -> u32 {
        let mut result = 0;
        for i in 0..10 {
            result += self.play[i].chaos();
        }
        result
    }

    pub fn hidden(&self) -> u32 {
        let mut result = 0;
        for i in 0..10 {
            result += self.play[i].hidden();
        }
        result
    }

    pub fn order(&self) -> u8 {
        let mut result = 0;
        for i in 0..10 {
            result += self.play[i].order();
        }
        result
    }

    pub fn chaos(&self) -> u32 {
        // first sum up inner pile chaos
        let mut result = self.pile_chaos();
        // per non-empty pile the chaos is at minimum 1
        // but if the pile is connected, we substract one
        // obvious wins are chaos 0
        for i in 0..10 {
            if self.play[i].count() == 0 {
                continue;
            }
            let c1 = self.play[i].at(0);
            if c1.rank() == 13 {
                result -= 1;
                continue;
            }
            for j in 0..10 {
                if j == i {
                    continue;
                }
                let jpile = &self.play[j];
                if jpile.count() == 0 {
                    continue;
                }
                // we don't need the suit here
                if c1.rank() == jpile.at(jpile.count() - 1).rank() - 1 {
                    result -= 1;
                    break;
                }
            }
        }
        let mut fp = self.free_plays();
        while fp > 0 && result > 0 {
            fp -= 1;
            result -= 1;
        }
        result
    }

    pub fn playable(&self) -> u32 {
        let mut result: u32 = 0;
        for i in 0..10 {
            result += self.play[i].playable() as u32;
        }
        result
    }

    pub fn under(&self) -> u32 {
        let mut result: u32 = 0;
        for i in 0..10 {
            result += self.play[i].under();
        }
        result
    }

    // how many talon cards will fall into sequence
    pub fn talon_matches(&self) -> u8 {
        if let Some(next_talon) = self.next_talon() {
            let mut result = 0;
            let next_talon = &self.talon[next_talon];
            for i in 0..10 {
                if self.play[i].count() == 0 {
                    continue;
                }
                let current = self.play[i].at(self.play[i].count() - 1);
                if next_talon.at(i).is_in_sequence_to(&current) {
                    result += 1;
                }
            }
            result
        } else {
            0
        }
    }

    pub fn get_moves_index(&self) -> usize {
        self.moves_index
    }

    pub fn set_moves_index(&mut self, index: usize) {
        self.moves_index = index;
    }

    pub fn apply_move(&self, m: &Move) -> Deck {
        let mut newdeck = self.clone();
        newdeck.moves[newdeck.moves_index] = *m;
        newdeck.moves_index += 1;

        if m.is_talon() {
            let from_pile = m.from();
            for to in 0..10 {
                let mut c = self.talon[from_pile].at(to);
                c.set_faceup(true);
                newdeck.set_play(to, self.play[to].add_card(c));
            }
            newdeck.set_talon(m.from(), Pile::empty());
            assert_eq!(newdeck.talon[m.from()].count(), 0);
            return newdeck;
        }

        if m.is_off() {
            let from_index = m.from();
            let from_pile = &self.play[from_index];
            let c = from_pile.at(from_pile.count() - 13);
            newdeck.set_off(self.off.add_card(c));
            newdeck.set_play(m.from(), self.play[m.from()].remove_cards(m.index()));
            return newdeck;
        }
        newdeck.set_play(
            m.to(),
            self.play[m.to()].copy_from(&self.play[m.from()], m.index()),
        );
        newdeck.set_play(m.from(), self.play[m.from()].remove_cards(m.index()));
        newdeck
    }

    pub fn shortest_pile(&self) -> usize {
        let mut result = self.play[0].count();
        for i in 1..10 {
            if result > self.play[i].count() {
                result = self.play[i].count();
            }
        }
        result
    }

    pub fn longest_sequence(&self) -> usize {
        let mut result = self.play[0].top_sequence_length();
        for i in 1..10 {
            let sl = self.play[i].top_sequence_length();
            if result < sl {
                result = sl;
            }
        }
        result
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
            self.play[i].remove_known(&mut cards);
        }
        for i in 0..5 {
            self.talon[i].remove_known(&mut cards);
        }
        let off = &self.off;
        for i in 0..off.count() {
            let suit = off.at(i).suit();
            for rank in 1..=13 {
                let c = Card::known(suit, rank);
                let index = cards.iter().position(|x| x.is_same_card(&c));
                if index.is_none() {
                    panic!("{} on off is already taken", c.to_string());
                }
                cards.remove(index.unwrap());
            }
        }
        if !cards.is_empty() {
            Card::shuffle(
                &mut cards,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            //println!("Cards {}", Card::vec_as_string(&cards));
        }
        for i in 0..10 {
            self.set_play(i, self.play[i].pick_unknown(&mut cards));
        }
        for i in 0..5 {
            self.set_talon(i, self.talon[i].pick_unknown(&mut cards));
        }
        if cards.len() > 0 {
            panic!("There are cards left: {}", Card::vec_as_string(&cards));
        }
    }

    pub fn shortest_path(
        &mut self,
        cap: usize,
        debug: bool,
        won_decks: Option<HashSet<u64>>,
    ) -> Option<i32> {
        let mut unvisited: [Vec<Rc<Deck>>; 6] = Default::default();
        unvisited[self.free_talons() as usize].push(Rc::new(self.clone()));
        // sort only the index
        let mut new_unvisited: Vec<WeightedMove> = Vec::new();
        let mut seen = HashSet::new();

        let mut depth: i32 = 0;
        let mut moves = vec![];

        loop {
            for i in 0..=5 {
                for deck in &unvisited[i] {
                    // reuse moves vector
                    deck.get_moves(&mut moves);

                    for m in &moves {
                        let newdeck = Rc::new(deck.apply_move(m));
                        let hash = newdeck.hash();
                        if !seen.contains(&hash) {
                            new_unvisited.push(WeightedMove::from(newdeck, hash));
                            seen.insert(hash);
                        }
                    }
                }
                unvisited[i].clear();
            }
            if new_unvisited.len() == 0 {
                break;
            }
            new_unvisited.sort_unstable();

            let mut iterator = new_unvisited.iter().rev();
            let mut printed = !debug;

            loop {
                if let Some(wm) = iterator.next() {
                    if wm.deck.is_won() {
                        self.moves = wm.deck.moves.clone();
                        self.moves_index = wm.deck.moves_index;
                        return Some(depth + 1);
                    }
                    if !printed {
                        println!(
                            "{}/{} {} {}",
                            depth,
                            new_unvisited.len(),
                            wm.deck.chaos(),
                            wm.deck.playable()
                        );
                        //println!("{}", wm.deck.to_string());
                        printed = true;
                    }
                    if unvisited[wm.talons as usize].len() < cap {
                        unvisited[wm.talons as usize].push(Rc::clone(&wm.deck));
                        if let Some(ref hashset) = won_decks {
                            if hashset.contains(&wm.hash) {
                                println!(
                                    "{} Found it at {}",
                                    depth,
                                    unvisited[wm.talons as usize].len() - 1
                                );
                                if depth == 34 {
                                    let mut count = 0;
                                    for v in &unvisited[wm.talons as usize] {
                                        println!(
                                            "{} {} {} {}\n{}",
                                            count,
                                            v.chaos(),
                                            v.playable() + v.in_off(),
                                            v.hash(),
                                            v.to_string()
                                        );
                                        count += 1;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    break;
                }
            }
            /*
                        let mut count = 0;
                        for v in &unvisited[5] {
                            println!(
                                "{} {} {}\n{}",
                                count,
                                v.chaos(),
                                v.playable(),
                                v.to_string()
                            );
                            count += 1;
                        }
            */
            new_unvisited.clear();
            depth += 1;
        }

        Some(-1 * depth)
    }

    pub fn top_card_unknown(&self, index: usize) -> bool {
        let pile = &self.play[index];
        if pile.count() == 0 {
            return false;
        }
        pile.at(pile.count() - 1).is_unknown()
    }

    pub fn replace_play_card(&mut self, play: usize, index: usize, c: &Card) {
        let mut c = Card::new(c.value());
        c.set_faceup(self.play[play].at(index).faceup());
        let new = self.play[play].replace_at(index, &c);
        self.play[play] = new;
    }

    // for test cases
    #[allow(dead_code)]
    pub fn compare(&self, other: &Deck) -> Ordering {
        let m1 = WeightedMove::from(Rc::new(self.clone()), self.hash());
        let m2 = WeightedMove::from(Rc::new(other.clone()), other.hash());
        m1.cmp(&m2)
    }
}

#[derive(Debug)]
pub struct DeltaMove {
    chaos: i32,
    under: i32,
    playable: i32,
    hidden: i32,
    order: i32,
    talon_matches: i32,
    off: bool,
    shorter: bool,
    longer: bool,
    fp: bool,
    ft: bool,
    //  deck: Deck,
    hash: u64,
    //  m: Move,
}

impl DeltaMove {
    pub fn any_good(&self) -> bool {
        (self.chaos > 0)
            || (self.under > 0)
            || (self.playable > 0)
            || (self.hidden > 0)
            || (self.order > 0)
            || (self.talon_matches > 0)
            || self.shorter
            || self.longer
            || self.off
            || self.fp
            || self.ft
    }

    pub fn new(orig: &Deck, m: &Move) -> Self {
        let newdeck = orig.apply_move(&m);
        DeltaMove {
            chaos: orig.pile_chaos() as i32 - newdeck.pile_chaos() as i32,
            under: orig.under() as i32 - newdeck.under() as i32,
            order: newdeck.order() as i32 - orig.order() as i32,
            off: newdeck.in_off() > orig.in_off(),
            playable: newdeck.playable() as i32 - orig.playable() as i32,
            fp: orig.free_plays() == 0 && newdeck.free_plays() > 0,
            shorter: orig.shortest_pile() > newdeck.shortest_pile(),
            longer: newdeck.longest_sequence() > orig.longest_sequence(),
            ft: newdeck.free_talons() > orig.free_talons(),
            hidden: orig.hidden() as i32 - newdeck.hidden() as i32,
            talon_matches: newdeck.talon_matches() as i32 - orig.talon_matches() as i32,
            hash: newdeck.hash(),
            //            deck: newdeck.clone(),
            //            m: *m,
        }
    }
}

impl PartialEq for DeltaMove {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for DeltaMove {}

impl PartialOrd for DeltaMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DeltaMove {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .chaos
            .cmp(&self.chaos)
            .then(other.under.cmp(&self.under))
    }
}

#[cfg(test)]
mod decktests {
    use super::*;

    #[test]
    fn parse() {
        let text = "Play0: KS..6S
Play1: |AH |4H QH..5H
Play2: |TH |2S |JS |KS |KS QH..JH 2H..AH
Play3: |6H 3H
Play4: |TH |2S |TS 9S..8S KH..JH TS 9H..6H 5S..3S 3S AS
Play5: |9S |9H 8H..7H
Play6: |7S |QS |KH |4H 3H..2H
Play7: |8S |JS |7S AS 5H..4H 2S..AS KH QS 6S..3S
Play8: 6S..4S 3H..AH
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
        let mut moves = vec![];
        deck.get_moves(&mut moves);
        // pick 2H+AH to move to 3H
        assert_eq!(
            moves,
            [
                Move::regular(0, 6, 7),
                Move::regular(2, 6, 8),
                Move::regular(2, 3, 7),
                Move::regular(2, 7, 7),
                Move::regular(4, 7, 16),
                Move::regular(7, 5, 10)
            ]
        );
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
        let mut moves = vec![];
        deck.get_moves(&mut moves);
        for m in &moves {
            println!("{}", deck.explain_move(m));
        }
        // pick 5H to move to 6H (among other, prune is gone)
        assert_eq!(
            moves,
            [
                Move::regular(0, 6, 7),
                Move::regular(2, 6, 8),
                Move::regular(2, 7, 7),
                Move::regular(4, 7, 16),
                Move::regular(4, 3, 13),
                Move::regular(9, 3, 0)
            ]
        );
    }

    #[test]
    fn dont_break_sequences() {
        let text = "Play0: KS QS JS TS 9S 8S 7S 6S
Play1: QS  
Play2: |KS |2S |JS |KS |JH QH 2H AH TH
Play3: |4H 3H 5H 6H
Play4: |TH |3S |TS 9S 8S KH QH JH TS 9H 8H 7H 6S 5S 4S 3S 2S AS
Play5: |7H |9H 8H 9S
Play6: |7S |KH |AH |4H 2H 3H
Play7: |JS |7S 6S 5H 4H 2S KH QS 6H 5S 4S 3S 8S AS
Play8: 6S 5S 4S 3H 2H AH
Play9: QH JH TH 9H 8H 7H 6H 5H
Deal0: 
Deal1: 
Deal2: 
Deal3: 
Deal4: 
Off: KS KH";
        let deck = Deck::parse(&text.to_string());
        let mut moves = vec![];
        deck.get_moves(&mut moves);
        for m in &moves {
            deck.explain_move(m);
        }
        // move 5S to 6S even as it splits a smaller sequence
        assert_eq!(moves, [Move::regular(4, 0, 13), Move::regular(5, 2, 3),]);
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
        let mut moves = vec![];
        deck.get_moves(&mut moves);
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
        let mut moves = vec![];
        deck.get_moves(&mut moves);
        // pick 9S to move to TS to uncover the other TS
        assert!(moves.contains(&Move::regular(4, 9, 3)));
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
Play7: |8S |JS |7S 6S 4S 5S
Play8: 6S 5H
Play9: TS JH KS
Deal0: 
Deal1: 
Deal2: 
Deal3: 
Deal4: 
Off: KS KH KH KS";
        let deck = Deck::parse(&text.to_string());
        let mut moves = vec![];
        deck.get_moves(&mut moves);
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
        assert_eq!(deck.playable(), 0);
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
        assert_eq!(deck.chaos(), 0);
        assert_eq!(deck.playable(), 13);
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
        assert_eq!(deck.shortest_path(10, false, None).expect("winnable"), 3);
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
        let res = deck.shortest_path(20, false, None);
        assert_eq!(res.expect("winnable"), 28);
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
        let res = deck.shortest_path(10, false, None);
        assert_eq!(res.expect("winnable"), 17);
        /*
        let win_moves = deck.win_moves();
        let mut mc = 0;
        for m in win_moves {
            mc += 1;
            print!("{}: ", mc);
            deck.explain_move(&m);
            deck = deck.apply_move(&m);
            //println!("{} {}\n{}", deck.chaos(), deck.playable(), deck.to_string());
        }
        assert!(false);*/
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
        let res = deck.shortest_path(3400, false, None);
        assert_eq!(res.expect("out of options"), -2);
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

    #[test]
    fn result_of_tap1() {
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
        let deck = Deck::parse(&text.to_string());
        assert_eq!(deck.result_of_tap(0), None);
        //assert_eq!(deck.result_of_tap(1), Some(Move::regular(1, 3, 0)));
        assert_eq!(deck.result_of_tap(2), Some(Move::regular(2, 6, 0)));
        assert_eq!(deck.result_of_tap(3), None);
        assert_eq!(deck.result_of_tap(4), Some(Move::regular(4, 8, 0)));
        //assert_eq!(deck.result_of_tap(5), Some(Move::regular(5, 0, 2)));
        //assert_eq!(deck.result_of_tap(6), Some(Move::regular(6, 0, 4)));
        //assert_eq!(deck.result_of_tap(7), Some(Move::regular(7, 0, 1)));
        assert_eq!(deck.result_of_tap(8), None);
    }

    #[test]
    fn result_of_tap2() {
        let text = "Play0: 7S 6S 5H 4H 3H 2H 2S JH TH 9H KS AH
        Play1: |XX |XX |9S |2H KH QS JS TS 9S 8S 7H
        Play2: |XX |XX |XX 4S
        Play3: |XX |XX |XX |XX |XX KH QH AS QH 3S 8S 7S 6S
        Play4: 6H
        Play5: JH TH 9H 8H 7H
        Play6: |JH QS 5H KS QS JS TS 9S 8S 7S 6S 5S TH 8H KS QS JS
        Play7: |3H |AH |3H |4S TS 9S 6H 9H KH QH AH 4H
        Play8: 3S JS 5H 5S 4S AS 3S
        Play9: KS 7H 7S 6S 5S 4S 3S 2S AS
        Deal0: 
        Deal1: 
        Deal2: 
        Deal3: 
        Deal4: 
        Off: KH";
        let deck = Deck::parse(&text.to_string());
        assert_eq!(deck.result_of_tap(4), Some(Move::regular(4, 5, 0)));
    }

    #[test]
    fn result_of_tap3() {
        let text = "Play0: KH QH
Play1: 
Play2: KH QH JH TH 9H 8H 7H 6H 5H 4H
Play3: |XX |XX |XX 4H
Play4: 2H AH
Play5: 4S 3S 2S AS
Play6: |JH QS 5H KS QS JS TS 9S 8S 7S 6S 5S TH 9H 8H
Play7: |3H |AH |3H |4S TS 9S 8S 7S 6S 5S
Play8: 3S JS
Play9: KS 7H 6H
Deal0: 
Deal1: 
Deal2: 
Deal3: 
Deal4: 
Off: KH KS KH KS";
        let deck = Deck::parse(&text.to_string());
        assert_eq!(deck.result_of_tap(6), None);
    }

    #[test]
    fn compare1() {
        let text = "Play0: 3S..AS
        Play1: JS..5S
        Play2: 
        Play3: KS..QS
        Play4: 4S
        Play5: JS..AS
        Play6: QH..AH
        Play7: 
        Play8: KS..QS
        Play9: KH
        Deal0: 
        Deal1: 
        Deal2: 
        Deal3: 
        Deal4: 
        Off: KS KH KS KH KH";
        let deck1 = Deck::parse(&text.to_string());
        let text = "Play0: 3H..AH
        Play1: JH..5H
        Play2: AS
        Play3: 
        Play4: 4H
        Play5: 
        Play6: QS
        Play7: JS..2S
        Play8: KH..QH
        Play9: KS
        Deal0: 
        Deal1: 
        Deal2: 
        Deal3: 
        Deal4: 
        Off: KS KH KS KH KH KS";
        let deck2 = Deck::parse(&text.to_string());
        assert_eq!(
            deck2.compare(&deck1),
            Ordering::Less,
            "deck1 is winnable in 5, deck2 in 6"
        );
    }

    #[test]
    fn compare2() {
        let text = "Play0: QH..JH
        Play1: 7S..2S
        Play2: KS..QS
        Play3: TH
        Play4: JS..8S
        Play5: AH
        Play6: AS
        Play7: 9H..6H
        Play8: KH
        Play9: 5H..2H
        Deal0: 
        Deal1: 
        Deal2: 
        Deal3: 
        Deal4: 
        Off: KS KH KH KS KS KH";
        let _deck1 = Deck::parse(&text.to_string());
        let text = "Play0: 
        Play1: 7S
        Play2: KS
        Play3: 
        Play4: JS..TS
        Play5: 5S..2S
        Play6: |AS |QS KH..AH
        Play7: 9S..8S
        Play8: 6S
        Play9: 
        Deal0: 
        Deal1: 
        Deal2: 
        Deal3: 
        Deal4: 
        Off: KS KH KH KS KS KH";
        let _deck2 = Deck::parse(&text.to_string());
        // deck1 is winnable in 8, deck2 in 7 - but it's hard to see
        //assert_eq!(deck2.compare(&deck1), Ordering::Greater);
    }

    #[test]
    fn talon_matches() {
        let text = "Play0: |TS |6S |5H |9H |8H JH..8H 9H
                    Play1: JS..TS KH
                    Play2: |QH |JH |4S |3H |AH 5H 4S..3S 9H QS
                    Play3: |4H |8S |8S |2H 4H..3H 2S JS
                    Play4: |QH |4H |6S |AH 6H 4S 3H..2H
                    Play5: |6H |7H |TH |AS 7S KS KS..QS
                    Play6: |6S |TH |8S 2H..AH JH 2S
                    Play7: |AS |2S |9S KS KH 5H
                    Play8: |7S |QH |JS |2H KS 7H 8S..4S
                    Play9: |QS |3H 7H
                    Deal0: 
                    Deal1: 
                    Deal2: |5H |7H |TH |TS |3S |5S |QS |7S |KH |KH
                    Deal3: |AS |3S |5S |8H |QH |JS |4H |6H |AH |6H
                    Deal4: |9S |5S |3S |AS |JH |9S |8H |9S |TS |2S
                    Off:";
        // now move 3h..2h from play4 to play8
        assert_eq!(Deck::parse(&text.to_string()).talon_matches(), 1);
        let text = "Play0: |TS |6S |5H |9H |8H JH..8H 9H
                    Play1: JS..TS KH
                    Play2: |QH |JH |4S |3H |AH 5H 4S..3S 9H QS
                    Play3: |4H |8S |8S |2H 4H..3H 2S JS
                    Play4: |QH |4H |6S |AH 6H 4S 
                    Play5: |6H |7H |TH |AS 7S KS KS..QS
                    Play6: |6S |TH |8S 2H..AH JH 2S
                    Play7: |AS |2S |9S KS KH 5H
                    Play8: |7S |QH |JS |2H KS 7H 8S..4S 3H..2H
                    Play9: |QS |3H 7H
                    Deal0: 
                    Deal1: 
                    Deal2: |5H |7H |TH |TS |3S |5S |QS |7S |KH |KH
                    Deal3: |AS |3S |5S |8H |QH |JS |4H |6H |AH |6H
                    Deal4: |9S |5S |3S |AS |JH |9S |8H |9S |TS |2S
                    Off:";
        assert_eq!(Deck::parse(&text.to_string()).talon_matches(), 2);
    }
}
