use std::fmt;

pub fn hello_world() {
    println!("Available marks:");
    println!("X: {}", Mark::X);
    println!("O: {}", Mark::O);
    println!("Blank: {}", Mark::Blank);
}

#[derive(Copy, Clone, Debug)]
enum Mark {
    X,
    O,
    Blank,
}

impl fmt::Display for Mark {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mark::X => write!(f, "X"),
            Mark::O => write!(f, "O"),
            Mark::Blank => write!(f, " "),
        }
    }
}
