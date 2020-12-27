use crate::card::Card;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::mem::MaybeUninit;
use std::rc::Rc;

pub struct PileTree {
    children: [Option<Box<PileTree>>; 256],
    pile: Rc<Pile>,
}

// the empty one is always there
static mut PILE_COUNT: u32 = 0;

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
        let bytes = [0; 104];
        PileTree {
            children: PileTree::nones(),
            pile: Rc::new(Pile {
                cards: bytes,
                count: 0,
                chaos: 0,
                id: 0,
                playable: 0,
            }),
        }
    }

    pub fn insert_pile(&mut self, cards: &[u8; 104], count: usize, index: usize) -> Rc<Pile> {
        if index == count && count == self.pile.count {
            return Rc::clone(&self.pile);
        }
        if self.children[cards[index] as usize].is_some() {
            let child = self.children[cards[index] as usize].as_deref_mut().unwrap();
            return child.insert_pile(cards, count, index + 1);
        }
        let pile_id = unsafe {
            PILE_COUNT += 1;
            if PILE_COUNT > (u16::MAX as u32) * 256 {
                panic!("We have too many piles!");
            }
            PILE_COUNT
        };
        let mut newpile = Pile {
            cards: *cards,
            count: index + 1,
            chaos: 0,
            playable: 0,
            id: pile_id,
        };
        newpile.chaos = newpile.calculate_chaos();
        newpile.playable = newpile.calculate_playable();

        self.children[cards[index] as usize] = Some(Box::new(PileTree {
            pile: Rc::new(newpile),
            children: PileTree::nones(),
        }));
        return self.insert_pile(cards, count, index);
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

pub struct Pile {
    id: u32,
    cards: [u8; 104],
    count: usize,
    chaos: u32,
    playable: u32,
}

pub struct PileManager {
    tree: PileTree,
    lock: RwLock<u8>,
}

static mut PM: Lazy<PileManager> = Lazy::new(|| PileManager::new());

impl PileManager {
    pub fn new() -> PileManager {
        let ret = PileManager {
            tree: PileTree::new(),
            lock: RwLock::new(1),
        };
        ret
    }

    /*
    pub fn output_tree() {
        unsafe {
            PM.tree.output("");
        }
    }*/

    fn or_insert(cards: &[u8; 104], count: usize) -> Rc<Pile> {
        unsafe {
            let _arr_lock = PM.lock.write();
            let pile = PM.tree.insert_pile(cards, count, 0);
            pile
        }
    }
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

impl Pile {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn empty() -> Rc<Pile> {
        unsafe {
            let _arr_lock = PM.lock.write();
            Rc::clone(&PM.tree.pile)
        }
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
        let mut cards = [0; 104];
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
        return Some(PileManager::or_insert(&cards, count));
    }

    pub fn to_string(&self) -> String {
        let mut strings = Vec::new();
        for i in 0..self.count {
            strings.push(self.at(i).to_string());
        }
        strings.join(" ")
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
        PileManager::or_insert(&newcards, newcount)
    }

    pub fn replace_at(&self, index: usize, c: &Card) -> Rc<Pile> {
        let mut newcards = self.cards.clone();
        newcards[index] = c.value();
        PileManager::or_insert(&newcards, self.count)
    }

    pub fn add_card(&self, card: Card) -> Rc<Pile> {
        let mut newcards = self.cards.clone();
        newcards[self.count] = card.value();
        let newcount = self.count + 1;
        PileManager::or_insert(&newcards, newcount)
    }

    pub fn copy_from(&self, orig_pile: &Pile, index: usize) -> Rc<Pile> {
        let mut newcards = self.cards.clone();
        let mut newcount = self.count;
        for i in index..orig_pile.count() {
            newcards[newcount] = orig_pile.at(i).value();
            newcount += 1;
        }
        PileManager::or_insert(&newcards, newcount)
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

    fn calculate_chaos(&self) -> u32 {
        let mut result = 0;
        let mut lastcard = Card::new(0);
        for i in 0..self.count {
            let current = self.at(i);
            if !current.faceup() {
                result += 3;
            } else {
                // first in stack
                if lastcard.value() == 0 {
                    result += 2;
                } else {
                    if lastcard.rank() == current.rank() + 1 {
                        if lastcard.suit() == current.suit() {
                            result += 1;
                        } else {
                            result += 2;
                        }
                    } else {
                        result += 2;
                    }
                }
            }
            lastcard = current;
        }
        result
    }

    #[allow(dead_code)]
    pub fn playable(&self) -> u32 {
        self.playable
    }

    fn calculate_playable(&self) -> u32 {
        /*if self.count < 1 {
            return 100;
        }*/
        if self.count < 2 {
            return self.count as u32;
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
                return self.count as u32;
            }
            index -= 1;
            topcard = current;
        }
        (self.count - index - 1) as u32
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
        PileManager::or_insert(&newcards, self.count)
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
}

#[cfg(test)]
mod piletests {
    use super::*;

    #[test]
    fn parse() {
        let pile1 = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        assert_eq!(pile1.to_string(), "|AS |3S |AS |6S |3H 8S");

        let pile1 = Pile::parse("|AS |3S |AS 8S..5s").expect("parsed");
        assert_eq!(pile1.to_string(), "|AS |3S |AS 8S 7S 6S 5S");
    }

    #[test]
    fn remove_cards() {
        let pile1 = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        let pile2 = pile1.remove_cards(5);
        assert_eq!(pile2.to_string(), "|AS |3S |AS |6S 3H");
        let pile3 = pile2.remove_cards(4);
        assert_eq!(pile3.to_string(), "|AS |3S |AS 6S");
        // we can repeat the operation with the same result
        assert_eq!(pile1.remove_cards(5).id, pile2.id);
    }

    #[test]
    fn copy_from() {
        let pile1 = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        let pile2 = Pile::parse("|TS 7S 6S").expect("parsed");
        let new_pile = pile1.copy_from(&pile2, 1);
        assert_eq!(new_pile.to_string(), "|AS |3S |AS |6S |3H 8S 7S 6S");
    }

    #[test]
    fn chaos() {
        let pile = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        assert_eq!(pile.chaos(), 17);
        let pile = Pile::parse("|TS 7S 6S").expect("parsed");
        assert_eq!(pile.chaos(), 6);
        let pile = Pile::parse("8S 7S 6S").expect("parsed");
        assert_eq!(pile.chaos(), 4);
        let pile = Pile::parse("8S 7H 6S").expect("parsed");
        assert_eq!(pile.chaos(), 6);
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
}
