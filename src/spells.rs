#[derive(Clone, Copy, Debug)]
pub enum Element {
	Earth,
	Water,
	Air,
	Fire,
}

pub struct Spell {
	first: Element,
	second: Element,
}

impl std::fmt::Debug for Spell {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}-{:?}", self.first, self.second)
	}
}

impl Spell {
	pub fn new(a: Element, b: Element) -> Spell {
		if (a as u8) < (b as u8) {
			Spell { first: a, second: b }
		} else {
			Spell { first: b, second: a }
		}
	}
}
