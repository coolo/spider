use rand::Rng;
use std::cmp::Ordering;
use std::io;

mod stack {
    struct Node<T> {
        data: T,
        next: Option<Box<Node<T>>>,
    }
    pub struct Stack<T> {
        top: Option<Box<Node<T>>>,
    }
    impl<T> Stack<T> {
        pub fn new() -> Self {
            Self { top: None }
        }
        pub fn push(&mut self, element: T) {
            let node = self.top.take();
            self.top = Some(Box::new(Node {
                data: element,
                next: node,
            }));
        }
        pub fn pop(&mut self) -> Option<T> {
            if let Some(node) = self.top.take() {
                self.top = node.next;
                Some(node.data)
            } else {
                None
            }
        }
    }
}

use stack::Stack;

fn main() {
    let mut a: Stack<i32> = Stack::new();
    a.push(1);
    a.push(2);
    println!("{:?}", a.pop());
    println!("{:?}", a.pop());
    println!("{:?}", a.pop());

    println!("Guess the number!");

    let secret_number = rand::thread_rng().gen_range(1, 101);

    loop {
        println!("Please input your guess.");

        let mut guess = String::new();

        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        println!("You guessed: {}", guess);
        let guess: u32 = match guess.trim().parse::<u32>() {
            Ok(num) => num,
            Err(_num) => continue,
        };

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}
