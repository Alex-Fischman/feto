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
	pub is_inverted: bool,
	// cast mods
	pub range: f32, // todo
	pub speed: f32,
	pub cost: f32, // todo
	// effect mods
	pub strength: f32, // todo
	pub duration: f32, // todo
	pub area: f32,     // todo
	// non-constant data
	pub dist_traveled: f32,
}

impl Spell {
	pub fn new(elements: &[Element]) -> Spell {
		let mut spell = Spell {
			element: elements[0],
			is_inverted: false,
			range: 1.0,
			speed: 1.0,
			cost: 1.0,
			strength: 1.0,
			duration: 1.0,
			area: 1.0,
			dist_traveled: 0.0,
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
}
