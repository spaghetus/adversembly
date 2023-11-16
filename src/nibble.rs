use std::ops::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
pub struct u4(u8);

impl From<u8> for u4 {
	fn from(value: u8) -> Self {
		Self(value)
	}
}

impl From<u4> for u8 {
	fn from(value: u4) -> Self {
		value.0
	}
}

impl u4 {
	pub fn combine(from: [u4; 2]) -> u8 {
		(from[0].0 << 4) | from[1].0
	}
	pub fn split(from: u8) -> [u4; 2] {
		[Self(from >> 4), Self(from & 0xF)]
	}
}

macro_rules! arith_trait {
	($me:ident, $trait1:ident, $trait2:ident, $fn1:ident, $fn2:ident) => {
		impl $trait1 for $me {
			type Output = $me;

			fn $fn1(self, rhs: Self) -> Self::Output {
				#[allow(clippy::suspicious_arithmetic_impl)]
				Self((self.0.$fn1(rhs.0)) & 0xF)
			}
		}

		impl $trait2 for $me {
			fn $fn2(&mut self, rhs: Self) {
				*self = self.$fn1(rhs);
			}
		}
	};
}

arith_trait!(u4, Add, AddAssign, add, add_assign);
arith_trait!(u4, Sub, SubAssign, sub, sub_assign);
arith_trait!(u4, Div, DivAssign, div, div_assign);
arith_trait!(u4, Mul, MulAssign, mul, mul_assign);
arith_trait!(u4, Rem, RemAssign, rem, rem_assign);
arith_trait!(u4, Shl, ShlAssign, shl, shl_assign);
arith_trait!(u4, Shr, ShrAssign, shr, shr_assign);
arith_trait!(u4, BitAnd, BitAndAssign, bitand, bitand_assign);
arith_trait!(u4, BitOr, BitOrAssign, bitor, bitor_assign);
arith_trait!(u4, BitXor, BitXorAssign, bitxor, bitxor_assign);

impl u4 {
	pub fn add_with_carry(self, rhs: u4) -> (u4, u4) {
		let sum = self.0 + rhs.0;
		(Self(sum & 0xF), Self(sum >> 4))
	}
}
