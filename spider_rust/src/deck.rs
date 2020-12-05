use crate::pile::Pile;
use std::collections::HashMap;

pub struct Deck {
    play: [u64; 10],
    talon: [u64; 5],
    off: u64,
}

pub struct Move {
    off: bool,
    talon: bool,
    from: i8,
    to: i8,
    index: u8,
}

impl Move {
    pub fn new() -> Move {
        Move {
            talon: false,
            off: false,
            from: -1,
            to: -1,
            index: 0,
        }
    }
    pub fn from_talon(from: i8) -> Move {
        Move {
            talon: true,
            off: false,
            from: from,
            to: -1,
            index: 0,
        }
    }
}
impl Deck {
    pub fn parse(contents: &String, pilemap: &mut HashMap<u64, Pile>) -> Deck {
        let mut newdeck = Deck {
            play: [0; 10],
            talon: [0; 5],
            off: 0,
        };
        let mut index = 0;
        for line in contents.lines() {
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
                    let parsed = Pile::parse(pile, pilemap);
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
        newdeck
    }

    pub fn to_string(&self, pilemap: &HashMap<u64, Pile>) -> String {
        let mut result = String::new();
        for i in 0..10 {
            result += &format!("Play{} {}\n", i, pilemap[&self.play[i]].to_string());
        }
        for i in 0..5 {
            result += &format!("Deck{} {}\n", i, pilemap[&self.talon[i]].to_string());
        }
        result += &format!("Off {}", pilemap[&self.off].to_string());
        result
    }

    pub fn get_moves(&self, pilemap: &HashMap<u64, Pile>) -> Vec<Move> {
        let mut vec = Vec::new();

        let mut next_talon: Option<i8> = None;
        for i in 0..5 {
            let talon = &pilemap[&self.talon[i]];
            if !talon.is_empty() {
                let index = i as i8;
                next_talon = Some(index);
                break;
            }
        }
        let one_is_empty = false;
        for _from in 0..10 {
            /*
            if (piles[from]->empty())
            {
                one_is_empty = true;
                continue;
            }

            int index = piles[from]->cardCount() - 1;
            Suit top_suit = piles[from]->at(index).suit;
            int top_rank = int(piles[from]->at(index).rank) - 1;

            while (index >= 0)
            {
                Card current = piles[from]->at(index);
                if (!current.faceup)
                    break;
                if (current.suit != top_suit)
                    break;
                if (top_rank + 1 != current.rank)
                    break;
                top_rank = piles[from]->at(index).rank;

                if (piles[from]->cardCount() - index == 13)
                {
                    ret.clear();
                    ret.append(Move());
                    ret.last().from = from;
                    ret.last().to = 0;
                    ret.last().off = true;
                    ret.last().index = index;
                    return ret;
                }
                bool moved_to_empty = false;
                for (int to = 0; to < 10; to++)
                {
                    if (to == from)
                        continue;
                    //qDebug() << "trying to move " << (piles[from]->cardCount() - index) << " from " << from << " to " << to;
                    int to_count = piles[to]->cardCount();
                    if (to_count > 0)
                    {
                        Card top_card = piles[to]->at(to_count - 1);
                        if (top_card.rank != top_rank + 1)
                            continue;
                    }
                    else if (moved_to_empty)
                    {
                        if (talons_done)
                            continue;
                    }
                    else
                    {
                        moved_to_empty = true;
                    }

                    ret.append(Move());
                    ret.last().from = from;
                    ret.last().to = to;
                    ret.last().index = index;
                }
                index--;
            } */
        }
        if !one_is_empty && next_talon.is_some() {
            vec.push(Move::from_talon(next_talon.unwrap()));
        }
        vec
    }

    pub fn explain_move(&self, m: &Move, pilemap: &HashMap<u64, Pile>) -> () {
        if m.talon {
            println!("Draw another talon");
            return;
        }
        if m.off {
            println!("Move a sequence from {} to the off", m.from + 1);
            return;
        }
        // happy casting to avoid storing every index as 64 bits
        let from_pile = m.from as usize;
        let from_pile = &pilemap[&self.play[from_pile]];
        let to_pile = m.to as usize;
        let to_pile = &pilemap[&self.play[to_pile]];
        let from_card = m.from as usize;
        let from_card = from_pile.at(from_card).to_string();
        let mut to_card = String::from("Empty");
        if to_pile.count() > 0 {
            let c = to_pile.at(to_pile.count() - 1);
            to_card = c.to_string();
        }
        let mut count = from_pile.count() as u8;
        count -= m.index;
        println!(
            "Move {} cards from {} to {} - {}->{}",
            count,
            m.from + 1,
            m.to + 1,
            from_card,
            to_card
        );
    }
}
