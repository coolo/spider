use crate::card::Card;
use fasthash::farm;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Pile {
    cards: [u8; 104],
    count: usize,
    chaos: u32,
    playable: u32,
}

pub struct PileManager {
    array: Vec<Arc<Pile>>,
    map: HashMap<u64, u32>,
    lock: RwLock<u8>,
}

static mut PM: Lazy<PileManager> = Lazy::new(|| PileManager::new());

impl PileManager {
    pub fn new() -> PileManager {
        PileManager {
            array: vec![],
            map: HashMap::new(),
            lock: RwLock::new(1),
        }
    }

    fn hash(cards: &[u8; 104]) -> u64 {
        farm::hash64(cards)
    }

    fn or_insert(cards: &[u8; 104], count: usize) -> u32 {
        let hash = PileManager::hash(&cards);
        unsafe {
            let _rlock = PM.lock.read();
            if let Some(pile) = PM.map.get(&hash) {
                return *pile;
            }
        }

        let mut new = Pile {
            cards: *cards,
            count: count,
            chaos: 0,
            playable: 0,
        };

        new.chaos = new.calculate_chaos();
        new.playable = new.calculate_playable();

        unsafe {
            let _arr_lock = PM.lock.write();
            PM.array.push(Arc::new(new));
            let index = (PM.array.len() - 1) as u32;
            PM.map.insert(hash, index);
            index
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
    pub fn get(index: u32) -> Arc<Pile> {
        unsafe {
            let _pm_lock = PM.lock.read();
            Arc::clone(&PM.array[index as usize])
        }
    }

    pub fn at(&self, index: usize) -> Card {
        Card::new(self.cards[index])
    }

    pub fn parse(s: &str) -> Option<u32> {
        let mut count = 0;
        let mut cards = [0; 104];
        for card_string in s.split(' ') {
            if card_string.is_empty() {
                continue;
            };
            match Card::parse(card_string) {
                None => {
                    println!("Card couldn't be parsed {}", card_string);
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
        let mut strings = Vec::new();
        for i in 0..self.count {
            strings.push(self.at(i).to_string());
        }
        strings.join(" ")
    }

    pub fn remove_cards(pile: u32, index: usize) -> u32 {
        // shadow
        let pile = Pile::get(pile);
        let mut newcards = pile.cards.clone();
        for i in index..pile.count {
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

    pub fn replace_at(pile: u32, index: usize, c: &Card) -> u32 {
        // shadow
        let pile = Pile::get(pile);
        let mut newcards = pile.cards.clone();
        newcards[index] = c.value();
        Pile::or_insert(&newcards, pile.count())
    }

    pub fn add_card(pile: u32, card: Card) -> u32 {
        let pile = Pile::get(pile);
        let mut newcards = pile.cards.clone();
        newcards[pile.count] = card.value();
        let newcount = pile.count + 1;
        Pile::or_insert(&newcards, newcount)
    }

    pub fn or_insert(cards: &[u8; 104], count: usize) -> u32 {
        PileManager::or_insert(cards, count)
    }

    pub fn copy_from(pile: u32, orig_pile: u32, index: usize) -> u32 {
        let pile = Pile::get(pile);
        let orig_pile = Pile::get(orig_pile);
        let mut newcards = pile.cards.clone();
        let mut newcount = pile.count;
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

    fn calculate_chaos(&self) -> u32 {
        let mut result = 0;
        let mut lastcard = Card::new(0);
        for i in 0..self.count {
            let current = self.at(i);
            if !current.faceup() {
                result += 10;
            } else {
                // first in stack
                if lastcard.value() == 0 {
                    result += 2;
                } else {
                    if lastcard.rank() == current.rank() + 1 {
                        if lastcard.suit() == current.suit() {
                            result += 1;
                        } else {
                            result += 5;
                        }
                    } else {
                        result += 3;
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

    pub fn pick_unknown(&self, cards: &mut Vec<Card>) -> u32 {
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
}

#[cfg(test)]
mod piletests {
    use super::*;

    #[test]
    fn parse() {
        let pile1 = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        assert_eq!(Pile::get(pile1).to_string(), "|AS |3S |AS |6S |3H 8S");
    }

    #[test]
    fn remove_cards() {
        let pile1 = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        let pile2 = Pile::remove_cards(pile1, 5);
        assert_eq!(Pile::get(pile2).to_string(), "|AS |3S |AS |6S 3H");
        let pile3 = Pile::remove_cards(pile2, 4);
        assert_eq!(Pile::get(pile3).to_string(), "|AS |3S |AS 6S");
        // we can repeat the operation with the same result
        assert_eq!(Pile::remove_cards(pile1, 5), pile2);
    }

    #[test]
    fn copy_from() {
        let pile1 = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        let pile2 = Pile::parse("|TS 7S 6S").expect("parsed");
        let new_pile = Pile::copy_from(pile1, pile2, 1);
        assert_eq!(
            Pile::get(new_pile).to_string(),
            "|AS |3S |AS |6S |3H 8S 7S 6S"
        );
    }

    #[test]
    fn chaos() {
        let pile = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        assert_eq!(Pile::get(pile).chaos(), 53);
        let pile = Pile::parse("|TS 7S 6S").expect("parsed");
        assert_eq!(Pile::get(pile).chaos(), 14);
        let pile = Pile::parse("8S 7S 6S").expect("parsed");
        assert_eq!(Pile::get(pile).chaos(), 4);
        let pile = Pile::parse("8S 7H 6S").expect("parsed");
        assert_eq!(Pile::get(pile).chaos(), 12);
    }

    #[test]
    fn playable() {
        let pile = Pile::parse("|AS |3S |AS |6S |3H 8S").expect("parsed");
        assert_eq!(Pile::get(pile).playable(), 1);
        let pile = Pile::parse("|8S 7S 6S").expect("parsed");
        assert_eq!(Pile::get(pile).playable(), 2);
        let pile = Pile::parse("8S 7S 6S").expect("parsed");
        assert_eq!(Pile::get(pile).playable(), 3);
        let pile = Pile::parse("8S 7H 6S").expect("parsed");
        assert_eq!(Pile::get(pile).playable(), 1);
        let pile = Pile::parse("8S").expect("parsed");
        assert_eq!(Pile::get(pile).playable(), 1);
        let pile = Pile::parse("").expect("parsed");
        assert_eq!(Pile::get(pile).playable(), 0);
    }
}
