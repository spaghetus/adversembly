use crate::nibble::u4;
use chumsky::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Datum {
	Literal(Vec<u4>),
	Label(String),
}

#[derive(Clone, Debug)]
pub enum Syntax {
	Filler(Datum),
	Label(String),
	Nop,
	Push(Datum),
	Pop,
	Add,
	Subtract,
	JumpNonZero,
	Call,
	Return,
	Load,
	Store,
	Halt,
	Comment(String),
}

macro_rules! simple_syntax {
	($fname:ident, $pattern:expr, $variant:ident) => {
		fn $fname() -> impl Parser<char, Syntax, Error = Simple<char>> {
			just($pattern).map(|_| Syntax::$variant)
		}
	};
}

fn literal() -> impl Parser<char, Vec<u4>, Error = Simple<char>> {
	just('x')
		.then(chumsky::text::digits(16).map(|d: String| {
			d.chars()
				.map(|c| c.to_digit(16).unwrap() as u8)
				.map(u4::from)
				.collect::<Vec<u4>>()
		}))
		.map(|(_, l)| l)
}

fn label_name() -> impl Parser<char, String, Error = Simple<char>> {
	just('\'').then(chumsky::text::ident()).map(|(_, s)| s)
}

fn datum() -> impl Parser<char, Datum, Error = Simple<char>> {
	choice((
		literal().map(Datum::Literal),
		label_name().map(Datum::Label),
	))
}

fn filler() -> impl Parser<char, Syntax, Error = Simple<char>> {
	just('.').then(datum()).map(|(_, d)| Syntax::Filler(d))
}

fn label() -> impl Parser<char, Syntax, Error = Simple<char>> {
	label_name().map(Syntax::Label)
}

simple_syntax!(nop, "?", Nop);

fn push() -> impl Parser<char, Syntax, Error = Simple<char>> {
	just("<").then(datum()).map(|(_, d)| Syntax::Push(d))
}

simple_syntax!(pop, ">", Pop);
simple_syntax!(add, "+", Add);
simple_syntax!(sub, "+", Subtract);
simple_syntax!(jnz, "j", JumpNonZero);
simple_syntax!(call, "c", Call);
simple_syntax!(ret, "r", Return);
simple_syntax!(load, "l", Load);
simple_syntax!(store, "s", Store);
simple_syntax!(halt, "!", Halt);

fn comment() -> impl Parser<char, Syntax, Error = Simple<char>> {
	just('(')
		.then(none_of(')').repeated())
		.then(just(')'))
		.map(|((_, s), _)| Syntax::Comment(s.into_iter().collect()))
}

pub fn program() -> impl Parser<char, Vec<Syntax>, Error = Simple<char>> {
	choice((
		filler(),
		label(),
		nop(),
		push(),
		pop(),
		add(),
		sub(),
		jnz(),
		call(),
		ret(),
		load(),
		store(),
		halt(),
		comment(),
	))
	.separated_by(chumsky::text::whitespace())
}

impl Datum {
	#[must_use]
	pub fn len(&self) -> usize {
		match self {
			Datum::Literal(d) => d.len(),
			Datum::Label(_) => 2,
		}
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

impl Syntax {
	#[must_use]
	pub fn len(&self) -> usize {
		match self {
			Syntax::Filler(d) => d.len(),
			Syntax::Label(_) | Syntax::Comment(_) => 0,
			Syntax::Push(d) => d.len() * 2,
			_ => 1,
		}
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn resolve_labels(program: &mut [Self]) {
		let mut address = 0;
		let mut element_table = HashMap::new();
		for element in program.iter() {
			if let Syntax::Label(label) = element {
				element_table.insert(label.to_owned(), address);
			}
			address += element.len();
		}
		for element in program.iter_mut() {
			match element {
				Syntax::Filler(d) | Syntax::Push(d) => {
					if let Datum::Label(l) = d.clone() {
						if let Some(addr) = element_table.get(&l) {
							*d = Datum::Literal(Vec::from(u4::split(*addr as u8)));
						} else {
							eprintln!("WARNING: Unresolved label {l}");
							*d = Datum::Literal(vec![u4::from(0); 2]);
						}
					}
				}
				_ => {}
			}
		}
	}

	pub fn compile(mut program: Vec<Self>) -> Vec<u4> {
		Self::resolve_labels(&mut program);
		program.into_iter().flat_map(Vec::from).collect()
	}
}

impl From<Syntax> for Vec<u4> {
	fn from(val: Syntax) -> Self {
		match val {
			Syntax::Filler(Datum::Literal(d)) => d,
			Syntax::Filler(Datum::Label(_)) => vec![0.into(); 2],
			Syntax::Label(_) => vec![],
			Syntax::Nop => vec![0.into()],
			Syntax::Push(Datum::Literal(d)) => {
				d.into_iter().flat_map(|d| vec![1.into(), d]).collect()
			}
			Syntax::Push(Datum::Label(_)) => vec![1.into(), 0.into(), 1.into(), 0.into()],
			Syntax::Pop => vec![2.into()],
			Syntax::Add => vec![3.into()],
			Syntax::Subtract => vec![4.into()],
			Syntax::JumpNonZero => vec![5.into()],
			Syntax::Call => vec![6.into()],
			Syntax::Return => vec![7.into()],
			Syntax::Load => vec![8.into()],
			Syntax::Store => vec![9.into()],
			Syntax::Halt => vec![15.into()],
			Syntax::Comment(_) => vec![],
		}
	}
}
