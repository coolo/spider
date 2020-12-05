use crate::pile::Pile;
use std::collections::HashMap;

pub struct Deck {
    play: [u64; 10],
    talon: [u64; 5],
    off: u64,
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
            let prefix: &str;
            match two.next() {
                None => {
                    break;
                }
                Some(string) => {
                    prefix = string;
                }
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
}
