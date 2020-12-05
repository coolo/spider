use crate::pile::Pile;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
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
    pub fn regular(from: usize, to: usize, index: usize) -> Move {
        Move {
            talon: false,
            off: false,
            from: from as i8,
            to: to as i8,
            index: index as u8,
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
    pub fn off(from: i8, index: u8) -> Move {
        Move {
            talon: false,
            off: true,
            from: from,
            to: 0,
            index: index,
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
        // an optimization - only move to first empty
        let mut one_is_empty = false;

        // translate ID into reference for quicker access
        let mut play_refs = Vec::new();
        for i in 0..10 {
            play_refs.push(&pilemap[&self.play[i]]);
        }

        for from in 0..10 {
            if play_refs[from].is_empty() {
                one_is_empty = true;
                continue;
            }
            let from_pile = play_refs[from];
            let mut index = from_pile.count() - 1;
            let top_card = from_pile.at(index);

            let top_suit = top_card.suit();
            let mut top_rank = top_card.rank() - 1;

            while index >= 0 {
                let current = from_pile.at(index);
                if !current.faceup() {
                    break;
                }
                if current.suit() != top_suit {
                    break;
                }
                if top_rank + 1 != current.rank() {
                    break;
                }
                top_rank = current.rank();

                if from_pile.count() - index == 13 {
                    // off move
                    vec.clear();
                    let from = from as i8;
                    let index = index as u8;
                    vec.push(Move::off(from, index));
                    return vec;
                }

                let mut moved_to_empty = false;

                for to in 0..10 {
                    if to == from {
                        continue;
                    }
                    let to_pile = play_refs[to];
                    let to_count = to_pile.count();

                    if to_count > 0 {
                        let top_card = to_pile.at(to_count - 1);
                        if top_card.rank() != top_rank + 1 {
                            continue;
                        }
                    } else if moved_to_empty {
                        if next_talon.is_none() {
                            continue;
                        };
                    } else {
                        moved_to_empty = true;
                    }
                    vec.push(Move::regular(from, to, index));
                }

                index -= 1;
            }
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
        let from_card = m.index as usize;
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

    pub fn apply_move(&self, m: &Move, mut pilemap: &mut HashMap<u64, Pile>) -> Deck {
        let mut newdeck = self.clone();

        if m.talon {
            let from_pile = m.from as usize;
            for to in 0..10 {
                let mut c = pilemap
                    .get(&self.talon[from_pile])
                    .expect("valid pile")
                    .at(to);
                c.set_faceup(true);
                newdeck.play[to] = Pile::add_card(self.play[to], c, &mut pilemap);
            }
            newdeck.talon[m.from as usize] = Pile::parse("", &mut pilemap).unwrap();
        }
        /*
        else if (m.off)
        {
            Card c = newone->piles[m.from]->at(newone->piles[m.from]->cardCount() - 13);
            newone->piles[15] = newone->piles[15]->newWithCard(c);
            newone->piles[m.from] = newone->piles[m.from]->remove(m.index);
        }
        else
        {
            newone->piles[m.to] = newone->piles[m.to]->copyFrom(newone->piles[m.from], m.index);
            newone->piles[m.from] = newone->piles[m.from]->remove(m.index);
            if (stop && m.index > 0 && newone->piles[m.from]->at(m.index - 1).unknown)
            {
                std::cout << "What's up?" << std::endl;
                std::string line;
                std::getline(std::cin, line);
                Card c(QString::fromStdString(line));
                newone->piles[m.from] = newone->piles[m.from]->replaceAt(m.index - 1, c);
                QFile file("tmp");
                file.open(QIODevice::WriteOnly);
                file.write(newone->toString().toUtf8());
                file.close();
                exit(1);
            }
        }
        */
        newdeck
    }
}
