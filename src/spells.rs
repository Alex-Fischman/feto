#[derive(Clone, Copy, Debug)]
pub enum Element {
	Earth,
	Water,
	Air,
	Fire,
	Acid,
	Pressure,
	Shock,
	Radiance,
	Life,
	Void,
}

pub struct Spell {
	pub element: Element,
	pub strength: f32,
	pub duration: f32,
	pub range: f32,
	pub speed: f32,
	pub area: f32,
	pub is_inverted: bool,
	pub cost: f32,
}

impl Spell {
	pub fn new(elements: &[Element]) -> Spell {
		let mut spell = Spell {
			element: elements[0],
			strength: 1.0,
			duration: 1.0,
			range: 1.0,
			speed: 1.0,
			area: 1.0,
			is_inverted: false,
			cost: 1.0,
		};
		for modifier in &elements[1..] {
			match modifier {
				Element::Earth => spell.speed = 0.0,
				Element::Water => spell.range = 0.0,
				Element::Air => spell.range += 1.0,
				Element::Fire => spell.speed += 1.0,
				Element::Acid => spell.duration += 1.0,
				Element::Pressure => {
					spell.area *= 0.5;
					spell.strength += 0.5;
				}
				Element::Shock => {
					spell.duration *= 0.5;
					spell.strength += 0.5;
				}
				Element::Radiance => spell.area += 1.0,
				Element::Life => spell.strength += 1.0,
				Element::Void => spell.is_inverted = !spell.is_inverted,
			}
			spell.cost += match modifier {
				Element::Void => 0.0,
				Element::Life => spell.cost,
				_ => 1.0,
			}
		}
		spell
	}

	pub fn activate(&mut self) {
		match (self.element, self.is_inverted) {
			_ => todo!(),
		}
	}
}
