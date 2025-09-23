use combine::EasyParser;
use combine_example::expression;

fn main() {
    let tests = vec!["42", "2 + 3", "2 + 3 * 4", "(2 + 3) * 4"];

    for test in tests {
        let result = expression().easy_parse(test);
        println!("{:?} => {:?}", test, result);
    }
}
