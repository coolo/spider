use std::fs;
mod card;
mod pile;
use pile::Pile;
use std::collections::HashMap;

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    // need to make this implicit
    let mut pilemap: HashMap<u64, Pile> = HashMap::new();
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
                let parsed = pile::Pile::parse(pile, &mut pilemap);
                println!("Pile {} {}", prefix, parsed.expect("Parsed"));
            }
        }
    }
    for pile in pilemap.values() {
        println!("Pile {}", pile.to_string());
    }
}
