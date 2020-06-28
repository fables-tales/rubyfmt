extern crate rubyfmt;
use std::io::{self, Read, Write};


fn main() {
    rubyfmt::rubyfmt_init();
    let args: Vec<String> = std::env::args().collect();
    println!("{:?}", args);
    if args.len() == 1 {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).expect("reading frmo stdin to not fail");
        let res = rubyfmt::format_buffer(buffer);
        write!(io::stdout(), "{}", res).expect("write works");
        io::stdout().flush().expect("flush works");
    }
}
