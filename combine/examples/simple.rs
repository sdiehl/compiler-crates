use combine::parser::char::{char, spaces};
use combine::{EasyParser, Parser};

fn main() {
    // Test basic number parsing
    let num_parser = combine::parser::char::digit().map(|c: char| c.to_digit(10).unwrap());

    println!("Parse '2': {:?}", num_parser.clone().easy_parse("2"));

    // Test spaces
    println!("Parse spaces in ' ': {:?}", spaces().easy_parse(" "));

    // Test number then space
    let mut parser = (num_parser.clone(), spaces());
    println!("Parse '2 ': {:?}", parser.easy_parse("2 "));

    // Test number, space, then char
    let mut parser2 = (num_parser.clone(), spaces(), char('+'));
    println!("Parse '2 +': {:?}", parser2.easy_parse("2 +"));
}
