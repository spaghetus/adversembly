use std::sync::Arc;

use smol::{
	channel::Receiver,
	lock::{Mutex, RwLock},
};

use crate::nibble::u4;

primitive_enum::primitive_enum! { Opcode u8 ;
	/// Nothing
	Nop = 0,
	/// Pushes the next nibble into the stack
	Push,
	/// Drops the top of the stack
	Pop,
	/// Pops the top of the stack, adds it to the next nibble, carrying as far back as it needs to
	Add,
	/// Pops the top of the stack, subtracts it from the next nibble, carrying
	Subtract,
	/// Pops one nibble for condition and two for address, if the condition is nonzero, jumps.
	JumpNonZero,
	/// Pops an address from the stack, jumps to it, and pushes our old instruction pointer plus one.
	Call,
	/// Pops an address from the stack and jumps to it.
	Return,
	/// Pops an address from the stack, pushes the nibble at that address.
	Load,
	/// Pops an address and a nibble from the stack, writes the nibble to that address.
	Store,
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
	pub memory: Arc<RwLock<[Option<u4>; 256]>>,
}

impl Vm {
	pub fn new(memory: Arc<RwLock<[Option<u4>; 256]>>) -> Self {
		Self {
			state: State::Running,
			last_read: 0.into(),
			stack_pointer: 0,
			instruction_pointer: 0,
			memory,
		}
	}
	/// Reads from memory, emulating open bus
	pub async fn read(&mut self, addr: u8) -> u4 {
		match self.memory.read().await[addr as usize] {
			Some(n) => {
				self.last_read = n;
				n
			}
			None => self.last_read,
		}
	}
	pub async fn is_open_bus(&mut self, addr: u8) -> bool {
		self.memory.read().await[addr as usize].is_none()
	}
	pub async fn write(&mut self, addr: u8, data: u4) {
		self.last_read = data;
		self.memory.write().await[addr as usize] = Some(data);
	}
	pub async fn push(&mut self, data: u4) {
		self.stack_pointer += 1;
		self.write(self.stack_pointer, data).await;
	}
	pub async fn pop(&mut self) -> u4 {
		self.stack_pointer -= 1;
		self.read(self.stack_pointer + 1).await
	}
	pub fn advance_ip(&mut self) {
		self.instruction_pointer = self.instruction_pointer.overflowing_add(1).0;
	}
	pub async fn cycle(&mut self) {
		if matches!(self.state, State::Halted) {
			return;
		}

		match Opcode::from(self.read(self.instruction_pointer).await.into()) {
			Some(op) => match op {
				Opcode::Nop => {
					self.advance_ip();
				}
				Opcode::Push => {
					self.advance_ip();
					let data = self.read(self.instruction_pointer).await;
					self.advance_ip();
					self.push(data).await;
				}
				Opcode::Pop => {
					self.advance_ip();
					let _ = self.pop().await;
				}
				Opcode::Add => {
					let mut carry = self.pop().await;
					let mut addr = self.stack_pointer;
					loop {
						let mut operand = self.read(addr).await;
						(operand, carry) = operand.add_with_carry(carry);
						self.write(addr, operand).await;
						if carry == 0.into() {
							break;
						}
						addr = addr.overflowing_sub(1).0;
					}
				}
				Opcode::Subtract => {
					let mut carry = u4::from(0) - self.pop().await;
					let mut addr = self.stack_pointer;
					loop {
						let mut operand = self.read(addr).await;
						(operand, carry) = operand.add_with_carry(carry);
						self.write(addr, operand).await;
						if carry == 0.into() {
							break;
						}
						addr = addr.overflowing_sub(1).0;
					}
				}
				Opcode::JumpNonZero => {
					let low = self.pop().await;
					let high = self.pop().await;
					let cond = self.pop().await;
					let addr = u4::combine([high, low]);
					if cond == 0.into() {
						self.advance_ip();
					} else {
						self.instruction_pointer = addr;
					}
				}
				Opcode::Call => {
					let mut low = self.pop().await;
					let mut high = self.pop().await;
					let old = self.instruction_pointer.overflowing_add(1).0;
					self.instruction_pointer = u4::combine([high, low]);
					[high, low] = u4::split(old);
					self.push(high).await;
					self.push(low).await;
				}
				Opcode::Return => {
					let low = self.pop().await;
					let high = self.pop().await;
					let addr = u4::combine([high, low]);
					self.instruction_pointer = addr;
				}
				Opcode::Load => {
					let low = self.pop().await;
					let high = self.pop().await;
					let addr = u4::combine([high, low]);
					let data = self.read(addr).await;
					self.push(data).await;
					self.advance_ip();
				}
				Opcode::Store => {
					let low = self.pop().await;
					let high = self.pop().await;
					let data = self.pop().await;
					let addr = u4::combine([high, low]);
					self.write(addr, data).await;
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
