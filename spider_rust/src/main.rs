struct Card {
    // 4 bits rank
    // 2 bits suit
    // 1 bit faceup
    // 1 bit unknown
    value: u8,
}

impl Card {
    fn faceup(&self) -> bool {
        self.value & (1 << 6) > 0
    }
    fn set_faceup(&mut self, face: bool) {
        if face {
            self.value = self.value | (1 << 6)
        } else {
            self.value = self.value & !(1 << 6)
        }
    }
    fn unknown(&self) -> bool {
        self.value & (1 << 7) > 0
    }
    fn set_unknown(&mut self, unknown: bool) {
        if unknown {
            self.value = self.value | (1 << 7)
        } else {
            self.value = self.value & !(1 << 7)
        }
    }
    fn rank(&self) -> u8 {
        self.value & 15
    }
    fn set_rank(&mut self, rank: u8) {
        self.value = (self.value & !15) + rank
    }
    fn suit(&self) -> u8 {
        (self.value >> 4) & 3
    }
    fn set_suit(&mut self, suit: u8) {
        let _rank = self.rank();
        self.value = self.value >> 4;
        self.value = (self.value & !3) + suit;
        self.value = (self.value << 4) + _rank;
    }
    fn new() -> Card {
        Card { value: 0 }
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
        assert!(!unknown.unknown());
        unknown.set_unknown(true);
        assert_eq!(unknown.value, 169);
        // stays
        assert_eq!(unknown.rank(), 9);
        assert_eq!(unknown.suit(), 2);
        assert!(unknown.unknown());
        unknown.set_unknown(false);
        assert!(!unknown.unknown());
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
}

fn main() {
    let c = Card::new();
    println!("Card face {}", c.faceup());
}
