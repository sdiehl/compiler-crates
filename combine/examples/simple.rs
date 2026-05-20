use combine::parser::char::{char, digit, spaces};
use combine::{EasyParser, Parser};

fn main() {
    let mut num_parser = digit().map(|c: char| c.to_digit(10).unwrap());

    println!("Parse '2': {:?}", num_parser.easy_parse("2"));

    println!("Parse spaces in ' ': {:?}", spaces().easy_parse(" "));

    let mut parser = (num_parser, spaces());
    println!("Parse '2 ': {:?}", parser.easy_parse("2 "));

    let mut parser2 = (num_parser, spaces(), char('+'));
    println!("Parse '2 +': {:?}", parser2.easy_parse("2 +"));
}
