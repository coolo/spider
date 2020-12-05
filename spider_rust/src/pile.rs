use crate::card::Card;

pub struct Pile {
    cards: [u8; 104],
    count: usize,
}

impl Pile {
    pub fn at(&self, index: usize) -> Card {
        Card::new(self.cards[index])
    }
    pub fn parse(s: &str) -> Option<Pile> {
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
        println!("Pile {}", new.to_string());
        return Some(new);
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
