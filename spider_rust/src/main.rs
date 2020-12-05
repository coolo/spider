//mod card;
//use card::Card;
use std::fs;

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    for line in contents.lines() {
        println!("Line {}", line);
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
            Some(pile) => println!("Pile {} {}", prefix, pile),
        }
    }
}
