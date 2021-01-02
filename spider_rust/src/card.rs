use std::fmt;

#[derive(PartialEq, Debug)]
pub struct Card {
    // 4 bits rank
    // 2 bits suit
    // 1 bit faceup
    // 1 bit unknown
    value: u8,
}

impl Card {
    #[inline]
    pub fn value(&self) -> u8 {
        self.value
    }

    #[inline]
    pub fn faceup(&self) -> bool {
        self.value & (1 << 6) > 0
    }

    #[inline]
    pub fn set_faceup(&mut self, face: bool) {
        if face {
            self.value = self.value | (1 << 6)
        } else {
            self.value = self.value & !(1 << 6)
        }
    }

    #[inline]
    pub fn is_unknown(&self) -> bool {
        self.value & (1 << 7) > 0
    }

    #[inline]
    pub fn set_unknown(&mut self, unknown: bool) {
        if unknown {
            self.value = self.value | (1 << 7)
        } else {
            self.value = self.value & !(1 << 7)
        }
    }

    #[inline]
    pub fn rank(&self) -> u8 {
        self.value & 15
    }

    #[inline]
    pub fn set_rank(&mut self, rank: u8) {
        self.value = (self.value & !15) + rank
    }

    #[inline]
    pub fn suit(&self) -> u8 {
        (self.value >> 4) & 3
    }

    #[inline]
    fn set_suit(&mut self, suit: u8) {
        let _rank = self.rank();
        self.value = self.value >> 4;
        self.value = (self.value & !3) + suit;
        self.value = (self.value << 4) + _rank;
    }
    pub fn new(value: u8) -> Card {
        Card { value: value }
    }

    pub fn known(suit: u8, rank: u8) -> Card {
        let mut card = Card::new(0);
        card.set_unknown(false);
        card.set_faceup(true);
        card.set_rank(rank);
        card.set_suit(suit);
        card
    }

    pub fn is_in_sequence_to(&self, next: &Card) -> bool {
        next.faceup() && next.suit() == self.suit() && next.rank() == self.rank() + 1
    }

    pub fn fits_on_top(&self, next: &Card) -> bool {
        next.faceup() && next.rank() == self.rank() + 1
    }

    pub fn opt_to_upper(c: Option<char>) -> Option<char> {
        match c {
            None => None,
            Some(c) => c.to_uppercase().nth(0),
        }
    }

    pub fn parse(token: &str) -> Option<Card> {
        let mut card = Card::new(0);
        let mut chars = token.chars();
        let mut current = chars.next();
        match current {
            Some('|') => {
                card.set_faceup(false);
                current = chars.next();
            }
            None => return None,
            _ => {
                card.set_faceup(true);
            }
        }
        match Card::opt_to_upper(current) {
            Some('A') => card.set_rank(1),
            Some('2') => card.set_rank(2),
            Some('3') => card.set_rank(3),
            Some('4') => card.set_rank(4),
            Some('5') => card.set_rank(5),
            Some('6') => card.set_rank(6),
            Some('7') => card.set_rank(7),
            Some('8') => card.set_rank(8),
            Some('9') => card.set_rank(9),
            Some('T') => card.set_rank(10),
            Some('J') => card.set_rank(11),
            Some('Q') => card.set_rank(12),
            Some('K') => card.set_rank(13),
            Some('X') => {
                card.set_unknown(true);
                match Card::opt_to_upper(chars.next()) {
                    Some('X') => {
                        return Some(card);
                    }
                    _ => {
                        return None;
                    }
                }
            }
            _ => {
                return None;
            }
        }
        match Card::opt_to_upper(chars.next()) {
            Some('S') => card.set_suit(0),
            Some('H') => card.set_suit(1),
            Some('C') => card.set_suit(2),
            Some('D') => card.set_suit(3),
            _ => {
                return None;
            }
        }
        if chars.count() > 0 {
            return None;
        }
        Some(card)
    }

    pub fn to_string(&self) -> String {
        let mut result;
        if self.is_unknown() {
            result = String::from("XX");
        } else {
            result = match self.rank() {
                1 => String::from("A"),
                2 => String::from("2"),
                3 => String::from("3"),
                4 => String::from("4"),
                5 => String::from("5"),
                6 => String::from("6"),
                7 => String::from("7"),
                8 => String::from("8"),
                9 => String::from("9"),
                10 => String::from("T"),
                11 => String::from("J"),
                12 => String::from("Q"),
                13 => String::from("K"),
                _ => panic!("broken card"),
            };
            result = result
                + match self.suit() {
                    0 => "S",
                    1 => "H",
                    2 => "C",
                    3 => "D",
                    _ => panic!("broken card"),
                };
        }
        if !self.faceup() {
            result = String::from("|") + &result;
        }
        return result;
    }

    pub fn vec_as_string(cards: &Vec<Card>) -> String {
        let mut res: String = String::from("[");
        let mut first = true;
        for c in cards {
            if first {
                first = false;
            } else {
                res.push_str(", ");
            }
            res.push_str(&c.to_string());
        }
        res + "]"
    }

    pub fn is_same_card(&self, other: &Card) -> bool {
        // we could also compare the bitmasked value, but this is easier to read
        self.rank() == other.rank() && self.suit() == other.suit()
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod cardtests {
    use super::*;

    #[test]
    fn faceup() {
        let mut facedown = Card { value: 41 };
        assert!(!facedown.faceup());
        assert_eq!(facedown.rank(), 9);
        assert_eq!(facedown.suit(), 2);
        facedown.set_faceup(true);
        assert_eq!(facedown.value, 105);
        // stays
        assert_eq!(facedown.rank(), 9);
        assert_eq!(facedown.suit(), 2);
        assert!(facedown.faceup());
        facedown.set_faceup(false);
        assert!(!facedown.faceup());
        assert_eq!(facedown.value, 41);
    }

    #[test]
    fn unknown() {
        let mut unknown = Card { value: 41 };
        assert!(!unknown.is_unknown());
        unknown.set_unknown(true);
        assert_eq!(unknown.value, 169);
        // stays
        assert_eq!(unknown.rank(), 9);
        assert_eq!(unknown.suit(), 2);
        assert!(unknown.is_unknown());
        unknown.set_unknown(false);
        assert!(!unknown.is_unknown());
        assert_eq!(unknown.value, 41);
    }

    #[test]
    fn rank() {
        let mut card = Card { value: 3 };
        assert_eq!(card.rank(), 3);
        card.set_rank(5);
        assert_eq!(card.rank(), 5);
        assert_eq!(card.value, 5);

        card.set_faceup(true);
        card.set_unknown(true);
        assert_eq!(card.value, 197);
        assert_eq!(card.rank(), 5);
        card.set_rank(3);
        assert_eq!(card.value, 195);
        assert_eq!(card.rank(), 3);
    }

    #[test]
    fn suit() {
        let mut card = Card { value: 3 };
        assert_eq!(card.suit(), 0);
        card.set_suit(2);
        assert_eq!(card.value, 35);
        assert_eq!(card.suit(), 2);

        card.set_faceup(true);
        card.set_unknown(true);
        assert_eq!(card.value, 227);
        assert_eq!(card.suit(), 2);
        card.set_suit(3);
        assert_eq!(card.value, 243);
        assert_eq!(card.suit(), 3);
    }

    #[test]
    fn parse() {
        let c = Card::parse("|AH");
        assert_eq!(c, Some(Card { value: 17 }));
        assert_eq!(c.unwrap().faceup(), false);
        let c = Card::parse("AH");
        assert_eq!(c, Some(Card { value: 81 }));
        assert_eq!(c.unwrap().faceup(), true);
        assert!(Card::parse("").is_none());
        assert!(Card::parse("|").is_none());
        assert!(Card::parse("AHx").is_none());
    }

    #[test]
    fn to_string() {
        let cards = vec!["|AH", "AH", "2S", "|XX", "XX", "TC", "7H", "KS", "|QH"];
        for card in cards.into_iter() {
            assert_eq!(Card::parse(card).unwrap().to_string(), card);
        }
    }

    #[test]
    fn test_format() {
        let card = Card::parse("|AH").unwrap();
        assert_eq!(format!("Test card: {}", card), "Test card: |AH");
    }

    #[test]
    fn is_in_sequence_to() {
        let card1 = Card::parse("2H").unwrap();
        let card2 = Card::parse("AH").unwrap();
        assert!(card2.is_in_sequence_to(&card1));
    }
}
