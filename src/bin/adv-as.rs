use adversembly::assembler::{program, Syntax};
use chumsky::Parser;
use std::io::{stdin, Read};

#[cfg(feature = "assembler")]
fn main() {
	let mut input = String::new();
	stdin()
		.lock()
		.read_to_string(&mut input)
		.expect("Read failure");
	let program = program().parse(input).expect("Parse failure");
	let program = Syntax::compile(program);
	let program = program
		.into_iter()
		.map(|n| char::from_digit(u8::from(n) as u32, 16).unwrap())
		.collect::<String>();
	println!("{program}");
}
