use crate::card::Card;
use once_cell::sync::Lazy;
use seahash;
use std::cmp::Ordering;
use std::mem::MaybeUninit;
use std::rc::Rc;

// the maximum number of cards in a pile (rounded on 8 bytes)
// 104 is the theoretical maximum but in real life 40 is already
// hard to construct - so pick something in between
pub const MAX_CARDS: usize = 64;

pub struct PileTree {
    children: [Option<Box<PileTree>>; 256],
    pile: Rc<Pile>,
}

impl PileTree {
    fn nones() -> [Option<Box<PileTree>>; 256] {
        // https://www.joshmcguigan.com/blog/array-initialization-rust/
        unsafe {
            let mut arr: [Option<Box<PileTree>>; 256] = MaybeUninit::uninit().assume_init();
            for item in &mut arr[..] {
                std::ptr::write(item, None);
            }
            arr
        }
    }

    pub fn new() -> PileTree {
        let bytes = [0; MAX_CARDS];
        PileTree {
            children: PileTree::nones(),
            pile: Rc::new(Pile {
                cards: bytes,
                count: 0,
                chaos: 0,
                playable: 0,
                under: 0,
                hidden: 0,
            }),
        }
    }

    #[inline]
    pub fn insert_pile(
        tree_p: &mut PileTree,
        cards: &[u8; MAX_CARDS],
        count: usize,
        index_p: usize,
    ) -> Rc<Pile> {
        let mut tree = tree_p;
        let mut index = index_p;
        loop {
            if index == count && count == tree.pile.count {
                return Rc::clone(&tree.pile);
            }
            if tree.children[cards[index] as usize].is_some() {
                let child = tree.children[cards[index] as usize].as_deref_mut().unwrap();
                tree = child;
                index = index + 1;
            } else {
                break;
            }
        }
        let mut newpile = Pile {
            cards: *cards,
            count: index + 1,
            chaos: 0,
            playable: 0,
            under: 0,
            hidden: 0,
        };
        newpile.chaos = newpile.calculate_chaos();
        newpile.playable = newpile.calculate_playable();
        newpile.under = newpile.calculate_under(0) as u32;
        newpile.hidden = newpile.calculate_hidden();

        tree.children[cards[index] as usize] = Some(Box::new(PileTree {
            pile: Rc::new(newpile),
            children: PileTree::nones(),
        }));
        return PileTree::insert_pile(tree, cards, count, index);
    }

    /*
    pub fn output(&self, prefix: &str) {
        println!("{}{}", prefix, self.pile.to_string());
        for child in &self.children {
            child.borrow().output(&(prefix.to_string() + "  "));
        }
    }
    */
}

static mut PILE_TREE: Lazy<PileTree> = Lazy::new(|| PileTree::new());

pub struct Pile {
    cards: [u8; MAX_CARDS],
    count: usize,
    chaos: u32,
    under: u32,
    playable: u8,
    hidden: u8,
}

impl PartialEq for Pile {
    fn eq(&self, other: &Self) -> bool {
        if self.count != other.count {
            return false;
        };
        for i in 0..self.count {
            if self.cards[i] != other.cards[i] {
                return false;
            }
        }
        true
    }
}
impl Eq for Pile {}

impl Ord for Pile {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = self.count.cmp(&other.count);
        if ord != Ordering::Equal {
            return ord;
        }
        for i in 0..self.count {
            let ord = self.cards[i].cmp(&other.cards[i]);
            if ord != Ordering::Equal {
                return ord;
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for Pile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Pile {
    pub fn or_insert(cards: &[u8; MAX_CARDS], count: usize) -> Rc<Pile> {
        unsafe { PileTree::insert_pile(&mut PILE_TREE, cards, count, 0) }
    }

    pub fn empty() -> Rc<Pile> {
        unsafe { Rc::clone(&PILE_TREE.pile) }
    }

    pub fn at(&self, index: usize) -> Card {
        Card::new(self.cards[index])
    }

    fn parse_sequence(s: &str) -> Option<Vec<Card>> {
        let split = s.split("..");
        let vec: Vec<&str> = split.collect();
        if vec.len() != 2 {
            return None;
        }
        let mut start = match Card::parse(vec[0]) {
            None => {
                return None;
            }
            Some(card) => card,
        };
        let end = match Card::parse(vec[1]) {
            None => {
                return None;
            }
            Some(card) => card,
        };
        if start.suit() != end.suit() || end.rank() >= start.rank() {
            return None;
        }
        let mut cards = vec![];
        cards.push(Card::new(start.value()));
        while start.rank() != end.rank() {
            start.set_rank(start.rank() - 1);
            cards.push(Card::new(start.value()));
        }
        return Some(cards);
    }

    pub fn parse(s: &str) -> Option<Rc<Pile>> {
        let mut count = 0;
        let mut cards = [0; MAX_CARDS];
        for card_string in s.split(' ') {
            if card_string.is_empty() {
                continue;
            }
            if card_string.contains("..") {
                if let Some(seq) = Pile::parse_sequence(card_string) {
                    for card in seq {
                        cards[count] = card.value();
                        count += 1;
                    }
                    continue;
                } else {
                    println!("Couldn't parse sequence {}", card_string);
                    return None;
                }
            }
            match Card::parse(card_string) {
                None => {
                    println!("Card couldn't be parsed '{}'", card_string);
                    return None;
                }
                Some(card) => {
                    cards[count] = card.value();
                    count += 1;
                }
            }
        }
        return Some(Pile::or_insert(&cards, count));
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        let mut startofseq: i32 = -1;
        for i in 0..self.count {
            let c = self.at(i);
            if i > 0 {
                let lastcard = self.at(i - 1);
                if lastcard.faceup()
                    && lastcard.suit() == c.suit()
                    && lastcard.rank() == c.rank() + 1
                {
                    continue;
                }
            }
            if startofseq != (i as i32) - 1 {
                result.push_str("..");
                result.push_str(&self.at(i - 1).to_string());
            }
            result.push(' ');
            result.push_str(&self.at(i).to_string());
            startofseq = i as i32;
        }
        if startofseq != (self.count as i32) - 1 {
            result.push_str("..");
            result.push_str(&self.at(self.count - 1).to_string());
        }
        result.trim_start().to_string()
    }

    pub fn remove_cards(&self, index: usize) -> Rc<Pile> {
        // shadow
        let mut newcards = self.cards.clone();
        for i in index..self.count {
            newcards[i] = 0;
        }
        let newcount = index;
        if newcount > 0 {
            let mut card = Card::new(newcards[newcount - 1]);
            card.set_faceup(true);
            newcards[newcount - 1] = card.value();
        }
        Pile::or_insert(&newcards, newcount)
    }

    pub fn replace_at(&self, index: usize, c: &Card) -> Rc<Pile> {
        let mut newcards = self.cards.clone();
        newcards[index] = c.value();
        Pile::or_insert(&newcards, self.count)
    }

    pub fn add_card(&self, card: Card) -> Rc<Pile> {
        let mut newcards = self.cards.clone();
        newcards[self.count] = card.value();
        let newcount = self.count + 1;
        Pile::or_insert(&newcards, newcount)
    }

    pub fn copy_from(&self, orig_pile: &Pile, index: usize) -> Rc<Pile> {
        let mut newcards = self.cards.clone();
        let mut newcount = self.count;
        for i in index..orig_pile.count() {
            newcards[newcount] = orig_pile.at(i).value();
            newcount += 1;
        }
        Pile::or_insert(&newcards, newcount)
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn count(&self) -> usize {
        self.count
    }

    pub fn chaos(&self) -> u32 {
        self.chaos
    }

    pub fn under(&self) -> u32 {
        self.under
    }

    pub fn hidden(&self) -> u32 {
        self.hidden as u32
    }

    pub fn calculate_hidden(&self) -> u8 {
        if self.count < 2 {
            return 0;
        }
        for i in 1..self.count {
            if self.at(i).faceup() {
                return (i - 1) as u8;
            }
        }
        self.count as u8
    }

    pub fn calculate_under(&self, ontop: usize) -> usize {
        if self.count < 2 {
            return self.count * ontop;
        }
        let count = self.sequence_of(self.at(self.count - 1).suit());
        let newpile = self.remove_cards(self.count - count);
        count * ontop + newpile.calculate_under(ontop + 1)
    }

    pub fn hash(&self, state: &mut seahash::State) {
        let ptr = self.cards.as_ptr();
        let mut offset: isize = 0;
        // we need to make sure MAX_CARDS is more
        let max_offset = (self.count + 1) as isize;

        loop {
            state.push(unsafe { (*(ptr.offset(offset) as *const u64)).to_le() });
            offset += 8;
            if offset >= max_offset {
                break;
            }
        }
    }

    fn calculate_chaos(&self) -> u32 {
        let mut result = 0;
        let mut lastcard = Card::new(0);
        for i in 0..self.count {
            let current = self.at(i);
            // first in stack
            if lastcard.value() == 0 {
                result += 1;
            } else {
                if !lastcard.faceup() {
                    result += 2;
                } else {
                    if lastcard.suit() != current.suit() {
                        result += 1;
                    }
                    if lastcard.rank() != current.rank() + 1 {
                        result += 1;
                    }
                }
            }
            lastcard = current;
        }
        result
    }

    #[allow(dead_code)]
    pub fn playable(&self) -> u8 {
        self.playable
    }

    fn calculate_playable(&self) -> u8 {
        /*if self.count < 1 {
            return 100;
        }*/
        if self.count < 2 {
            return self.count as u8;
        }
        let mut index = self.count - 1;
        let mut topcard = self.at(index);
        index -= 1;
        loop {
            let current = self.at(index);
            if current.suit() != topcard.suit()
                || !current.faceup()
                || current.rank() != topcard.rank() + 1
            {
                break;
            }
            if index == 0 {
                return self.count as u8;
            }
            index -= 1;
            topcard = current;
        }
        (self.count - index - 1) as u8
    }

    pub fn remove_known(&self, cards: &mut Vec<Card>) {
        for i in 0..self.count {
            let c = self.at(i);
            if c.is_unknown() {
                continue;
            }
            match cards.iter().position(|x| x.is_same_card(&c)) {
                Some(index) => {
                    cards.remove(index);
                }
                None => {
                    panic!("The card {} is not in {}", c, Card::vec_as_string(cards));
                }
            }
        }
    }

    pub fn pick_unknown(&self, cards: &mut Vec<Card>) -> Rc<Pile> {
        let mut newcards = self.cards.clone();
        for i in 0..self.count {
            let c = self.at(i);
            if !c.is_unknown() {
                continue;
            }
            let mut firstpick = cards.pop().expect("Enough cards");
            firstpick.set_faceup(false);
            firstpick.set_unknown(true);
            newcards[i] = firstpick.value();
        }
        Pile::or_insert(&newcards, self.count)
    }

    pub fn sequence_of(&self, suit: u8) -> usize {
        let mut index = self.count();
        if index == 0 {
            return 0;
        }
        index -= 1;
        let mut top_card = self.at(index);
        if top_card.suit() != suit {
            return 0;
        }
        while index > 0 && top_card.is_in_sequence_to(&self.at(index - 1)) {
            index -= 1;
            top_card = self.at(index);
        }
        self.count() - index
    }

    pub fn top_sequence_length(&self) -> usize {
        if self.count < 2 {
            return self.count;
        }
        self.sequence_of(self.at(self.count - 1).suit())
    }
}

#[cfg(test)]
mod piletests {
    use super::*;

    #[test]
    fn parse() {
        let pile1 = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        assert_eq!(pile1.to_string(), "|AS |3S |AS |6S |3H 8S");

        let pile1 = Pile::parse("|AS |3S |AS 4S 3S 2S 8S..5s").expect("parsed");
        assert_eq!(pile1.to_string(), "|AS |3S |AS 4S..2S 8S..5S");
    }

    #[test]
    fn remove_cards() {
        let pile1 = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        let pile2 = pile1.remove_cards(5);
        assert_eq!(pile2.to_string(), "|AS |3S |AS |6S 3H");
        let pile3 = pile2.remove_cards(4);
        assert_eq!(pile3.to_string(), "|AS |3S |AS 6S");
        // we can repeat the operation with the same result
        assert_eq!(pile1.remove_cards(5).to_string(), pile2.to_string());
    }

    #[test]
    fn copy_from() {
        let pile1 = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        let pile2 = Pile::parse("|TS 7S 6S").expect("parsed");
        let new_pile = pile1.copy_from(&pile2, 1);
        assert_eq!(new_pile.to_string(), "|AS |3S |AS |6S |3H 8S..6S");
    }

    #[test]
    fn chaos() {
        let pile = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        assert_eq!(pile.chaos(), 11);
        let pile = Pile::parse("|TS 7S 6S").expect("parsed");
        assert_eq!(pile.chaos(), 3);
        let pile = Pile::parse("8S 7S 6S").expect("parsed");
        assert_eq!(pile.chaos(), 1);
        let pile = Pile::parse("8S 7H 6S").expect("parsed");
        assert_eq!(pile.chaos(), 3);
        let pile = Pile::parse("8S 6S 7H").expect("parsed");
        assert_eq!(pile.chaos(), 4);
        let pile = Pile::parse("8S 7H 6H").expect("parsed");
        assert_eq!(pile.chaos(), 2);
    }

    #[test]
    fn playable() {
        let pile = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        assert_eq!(pile.playable(), 1);
        let pile = Pile::parse("|8S 7S 6S").expect("parsed");
        assert_eq!(pile.playable(), 2);
        let pile = Pile::parse("8S 7S 6S").expect("parsed");
        assert_eq!(pile.playable(), 3);
        let pile = Pile::parse("8S 7H 6S").expect("parsed");
        assert_eq!(pile.playable(), 1);
        let pile = Pile::parse("8S").expect("parsed");
        assert_eq!(pile.playable(), 1);
        let pile = Pile::parse("").expect("parsed");
        assert_eq!(pile.playable(), 0);
    }

    #[test]
    fn sequence_of() {
        let pile = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        assert_eq!(pile.sequence_of(0), 1);
        let pile = Pile::parse("|8S 7S 6S").expect("parsed");
        assert_eq!(pile.sequence_of(0), 2);
        assert_eq!(pile.sequence_of(1), 0);
    }

    #[test]
    fn undertaken() {
        // 4h..3h is free, 8s..6s are 3 cards with cost 1 (3), 3h is 1 card with cost 2 (3+2),
        // 6s is 1 one card with cost 3 (3+2+3) - so we expect 8
        let pile = Pile::parse("|6S |3H 8S..6s 4h..3h").expect("parsed");
        assert_eq!(pile.calculate_under(0), 8);
        let pile = Pile::parse("|8S 7S..6S").expect("parsed");
        assert_eq!(pile.calculate_under(0), 1);
    }
}
