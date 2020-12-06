use crate::card::Card;
use fasthash::farm;
use std::collections::HashMap;

pub struct Pile {
    cards: [u8; 104],
    count: usize,
    chaos: u32,
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
    pub fn at(&self, index: usize) -> Card {
        Card::new(self.cards[index])
    }
    fn hash(cards: &[u8; 104], count: usize) -> u64 {
        for i in count..104 {
            assert!(cards[i] == 0);
        }
        farm::hash64(cards)
    }

    pub fn parse(s: &str, hashmap: &mut HashMap<u64, Pile>) -> Option<u64> {
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
        return Some(Pile::or_insert(&cards, count, hashmap));
    }

    fn or_insert(cards: &[u8; 104], count: usize, hashmap: &mut HashMap<u64, Pile>) -> u64 {
        let hash = Pile::hash(&cards, count);
        match hashmap.get(&hash) {
            None => {
                let mut new = Pile {
                    cards: *cards,
                    count: count,
                    chaos: 0,
                };
                new.chaos = new.calculate_chaos();
                hashmap.entry(hash).or_insert(new);
            }
            _ => (),
        }
        hash
    }

    pub fn to_string(&self) -> String {
        let mut strings = Vec::new();
        for i in 0..self.count {
            strings.push(self.at(i).to_string());
        }
        strings.join(" ")
    }

    pub fn remove_cards(pile: u64, index: usize, pilemap: &mut HashMap<u64, Pile>) -> u64 {
        // shadow
        let pile = pilemap.get(&pile).expect("valid pile");
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
        Pile::or_insert(&newcards, newcount, pilemap)
    }

    pub fn add_card(pile: u64, card: Card, pilemap: &mut HashMap<u64, Pile>) -> u64 {
        let pile = pilemap.get(&pile).expect("valid pile");
        let mut newcards = pile.cards.clone();
        newcards[pile.count] = card.value();
        let newcount = pile.count + 1;
        Pile::or_insert(&newcards, newcount, pilemap)
    }

    pub fn copy_from(
        pile: u64,
        orig_pile: u64,
        index: usize,
        pilemap: &mut HashMap<u64, Pile>,
    ) -> u64 {
        let pile = pilemap.get(&pile).expect("valid pile");
        let orig_pile = pilemap.get(&orig_pile).expect("valid pile");
        let mut newcards = pile.cards.clone();
        let mut newcount = pile.count;
        for i in index..orig_pile.count() {
            newcards[newcount] = orig_pile.at(i).value();
            newcount += 1;
        }
        Pile::or_insert(&newcards, newcount, pilemap)
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
}

#[cfg(test)]
mod piletests {
    use super::*;

    #[test]
    fn parse() {
        let mut hashmap = HashMap::new();
        let pile1 = Pile::parse("|AS |3S |AS |6S |3H 8S", &mut hashmap).expect("parsed");
        assert_eq!(
            hashmap.get(&pile1).expect("valid pile").to_string(),
            "|AS |3S |AS |6S |3H 8S"
        );
    }

    #[test]
    fn remove_cards() {
        let mut hashmap = HashMap::new();
        let pile1 = Pile::parse("|AS |3S |AS |6S |3H 8S", &mut hashmap).expect("parsed");
        let pile2 = Pile::remove_cards(pile1, 5, &mut hashmap);
        assert_eq!(
            hashmap.get(&pile2).expect("valid pile").to_string(),
            "|AS |3S |AS |6S 3H"
        );
        let pile3 = Pile::remove_cards(pile2, 4, &mut hashmap);
        assert_eq!(
            hashmap.get(&pile3).expect("valid pile").to_string(),
            "|AS |3S |AS 6S"
        );
        // we can repeat the operation with the same result
        assert_eq!(Pile::remove_cards(pile1, 5, &mut hashmap), pile2);
    }

    #[test]
    fn copy_from() {
        let mut hashmap = HashMap::new();
        let pile1 = Pile::parse("|AS |3S |AS |6S |3H 8S", &mut hashmap).expect("parsed");
        let pile2 = Pile::parse("|TS 7S 6S", &mut hashmap).expect("parsed");
        let new_pile = Pile::copy_from(pile1, pile2, 1, &mut hashmap);
        assert_eq!(
            hashmap.get(&new_pile).expect("valid pile").to_string(),
            "|AS |3S |AS |6S |3H 8S 7S 6S"
        );
    }

    #[test]
    fn chaos() {
        let mut hashmap = HashMap::new();
        let pile = Pile::parse("|AS |3S |AS |6S |3H 8S", &mut hashmap).expect("parsed");
        assert_eq!(hashmap.get(&pile).expect("valid pile").chaos(), 53);
        let pile = Pile::parse("|TS 7S 6S", &mut hashmap).expect("parsed");
        assert_eq!(hashmap.get(&pile).expect("valid pile").chaos(), 14);
        let pile = Pile::parse("8S 7S 6S", &mut hashmap).expect("parsed");
        assert_eq!(hashmap.get(&pile).expect("valid pile").chaos(), 4);
        let pile = Pile::parse("8S 7H 6S", &mut hashmap).expect("parsed");
        assert_eq!(hashmap.get(&pile).expect("valid pile").chaos(), 12);
    }
}
