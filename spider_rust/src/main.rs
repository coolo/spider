use std::fs;
mod card;
mod pile;

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
            Some(pile) => {
                println!("Pile {} {}", prefix, pile);
                let parsed = pile::Pile::parse(pile);
                assert!(parsed.is_some());
            }
        }
    }
}
