use crate::nibble::u4;
use std::{cell::RefCell, rc::Rc};

primitive_enum::primitive_enum! { Opcode u8 ;
	/// Nothing
	Nop = 0,
	/// Pushes the next nibble into the stack
	Push = 1,
	/// Drops the top of the stack
	Pop = 2,
	/// Pops the top of the stack, adds it to the next nibble, carrying as far back as it needs to
	Add = 3,
	/// Pops the top of the stack, subtracts it from the next nibble, carrying
	Subtract = 4,
	/// Pops two nibbles for address and one for condition, if the condition is nonzero, jumps.
	JumpNonZero = 5,
	/// Pops an address from the stack, jumps to it, and pushes our old instruction pointer plus one.
	Call = 6,
	/// Pops an address from the stack and jumps to it.
	Return = 7,
	/// Pops an address from the stack, pushes the nibble at that address.
	Load = 8,
	/// Pops an address and a nibble from the stack, writes the nibble to that address.
	Store = 9,
	Halt = 15
}

#[derive(Clone)]
pub enum State {
	Running,
	Halted,
}

#[derive(Clone)]
pub struct Vm {
	pub state: State,
	pub last_read: u4,
	pub stack_pointer: u8,
	pub instruction_pointer: u8,
	// pub memory: Rc<RefCell<[Option<u4>; 256]>>,
}

impl Vm {
	pub fn new() -> Self {
		Self {
			state: State::Running,
			last_read: 0.into(),
			stack_pointer: 0,
			instruction_pointer: 0,
			// memory,
		}
	}
	/// Reads from memory, emulating open bus
	pub fn read(&mut self, memory: &[Option<u4>; 256], addr: u8) -> u4 {
		match memory[addr as usize] {
			Some(n) => {
				self.last_read = n;
				n
			}
			None => self.last_read,
		}
	}
	pub fn is_open_bus(&mut self, memory: &[Option<u4>; 256], addr: u8) -> bool {
		memory[addr as usize].is_none()
	}
	pub fn write(&mut self, memory: &mut [Option<u4>; 256], addr: u8, data: u4) {
		self.last_read = data;
		memory[addr as usize] = Some(data);
	}
	pub fn push(&mut self, memory: &mut [Option<u4>; 256], data: u4) {
		self.stack_pointer = self.stack_pointer.overflowing_add(1).0;
		self.write(memory, self.stack_pointer, data);
	}
	pub fn pop(&mut self, memory: &[Option<u4>; 256]) -> u4 {
		let val = self.read(memory, self.stack_pointer);
		self.stack_pointer = self.stack_pointer.overflowing_sub(1).0;
		val
	}
	pub fn advance_ip(&mut self) {
		self.instruction_pointer = self.instruction_pointer.overflowing_add(1).0;
	}
	pub fn cycle(&mut self, memory: &mut [Option<u4>; 256]) {
		if matches!(self.state, State::Halted) {
			return;
		}

		match Opcode::from(self.read(memory, self.instruction_pointer).into()) {
			Some(op) => match op {
				Opcode::Nop => {
					self.advance_ip();
				}
				Opcode::Push => {
					self.advance_ip();
					let data = self.read(memory, self.instruction_pointer);
					self.advance_ip();
					self.push(memory, data);
				}
				Opcode::Pop => {
					self.advance_ip();
					let _ = self.pop(memory);
				}
				Opcode::Add => {
					self.advance_ip();
					let mut carry = self.pop(memory);
					let mut addr = self.stack_pointer;
					loop {
						let mut operand = self.read(memory, addr);
						(operand, carry) = operand.add_with_carry(carry);
						self.write(memory, addr, operand);
						if carry == 0.into() {
							break;
						}
						addr = addr.overflowing_sub(1).0;
					}
				}
				Opcode::Subtract => {
					let mut carry = u4::from(0) - self.pop(memory);
					let mut addr = self.stack_pointer;
					loop {
						let mut operand = self.read(memory, addr);
						(operand, carry) = operand.add_with_carry(carry);
						self.write(memory, addr, operand);
						if carry == 0.into() {
							break;
						}
						addr = addr.overflowing_sub(1).0;
					}
				}
				Opcode::JumpNonZero => {
					let low = self.pop(memory);
					let high = self.pop(memory);
					let cond = self.pop(memory);
					let addr = u4::combine([high, low]);
					if cond == 0.into() {
						self.advance_ip();
					} else {
						self.instruction_pointer = addr;
					}
				}
				Opcode::Call => {
					let mut low = self.pop(memory);
					let mut high = self.pop(memory);
					let old = self.instruction_pointer.overflowing_add(1).0;
					self.instruction_pointer = u4::combine([high, low]);
					[high, low] = u4::split(old);
					self.push(memory, high);
					self.push(memory, low);
				}
				Opcode::Return => {
					let low = self.pop(memory);
					let high = self.pop(memory);
					let addr = u4::combine([high, low]);
					self.instruction_pointer = addr;
				}
				Opcode::Load => {
					let low = self.pop(memory);
					let high = self.pop(memory);
					let addr = u4::combine([high, low]);
					let data = self.read(memory, addr);
					self.push(memory, data);
					self.advance_ip();
				}
				Opcode::Store => {
					let low = self.pop(memory);
					let high = self.pop(memory);
					let data = self.pop(memory);
					let addr = u4::combine([high, low]);
					self.write(memory, addr, data);
					self.advance_ip();
				}
				Opcode::Halt => {
					self.state = State::Halted;
				}
			},
			None => {
				self.state = State::Halted;
			}
		}
	}
}

impl Default for Vm {
	fn default() -> Self {
		Self::new()
	}
}

pub fn load_memory_from_file(data: String) -> [Option<u4>; 256] {
	let mut memory = [None; 256];
	for (n, c) in data
		.chars()
		.filter(|c| c.is_alphanumeric() || *c == ' ')
		.enumerate()
		.take(256)
	{
		if c == ' ' {
			continue;
		}
		let Some(digit) = c.to_digit(16).map(|v| v as u8) else {
			panic!("Character {n} at position {n} is not a legal digit")
		};
		memory[n] = Some(digit.into());
	}
	memory
}
