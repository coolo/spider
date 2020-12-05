use crate::card::Card;

pub struct Pile {
    cards: [u8; 104],
    count: usize,
}

impl Pile {
    pub fn parse(s: &str) -> Option<Pile> {
        let mut new = Pile {
            count: 0,
            cards: [0; 104],
        };
        for card_string in s.split(' ') {
            println!("Card '{}'", card_string);
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
        return Some(new);
    }
}
