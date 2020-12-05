use crate::card::Card;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;

pub struct Pile {
    cards: [u8; 104],
    count: usize,
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
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write_usize(self.count);
        hasher.write(&self.cards);
        hasher.finish()
    }

    pub fn parse(s: &str, hashmap: &mut HashMap<u64, Pile>) -> Option<u64> {
        let mut new = Pile {
            count: 0,
            cards: [0; 104],
        };
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
                    new.cards[new.count] = card.value();
                    new.count += 1;
                }
            }
        }
        let hash = new.hash();
        match hashmap.get(&hash) {
            None => {
                let unmut = new;
                hashmap.entry(hash).or_insert(unmut);
            }
            _ => (),
        }
        return Some(hash);
    }

    pub fn to_string(&self) -> String {
        let mut strings = Vec::new();
        for i in 0..self.count {
            strings.push(self.at(i).to_string());
        }
        strings.join(" ")
    }
}

#[cfg(test)]
mod piletests {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(
            Pile::parse("|AS |3S |AS |6S |3H 8S")
                .expect("parsed")
                .to_string(),
            "|AS |3S |AS |6S |3H 8S"
        );
    }
}
