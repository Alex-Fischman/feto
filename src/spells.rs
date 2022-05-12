use crate::object::{Object, Shape};
use crate::render::{Color, Renderable, Vertex};
use crate::vector::Vector;

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
	pub object: Object,
	pub element: Element,
	pub strength: f32,
	pub duration: f32,
	pub range: f32,
	pub speed: f32,
	pub area: f32,
	pub is_inverted: bool,
	pub is_trap: bool,
	pub cost: f32,
}

impl Spell {
	pub fn new(pos: Vector, vel: Vector, elements: &[Element]) -> Option<Spell> {
		if elements.is_empty() {
			return None;
		}
		let mut stats = Spell {
			object: Object { pos, vel, shape: Shape::Aabb(Vector::new(0.1, 0.1)) },
			element: elements[0],
			strength: 1.0,
			duration: 1.0,
			range: 1.0,
			speed: 1.0,
			area: 1.0,
			is_inverted: false,
			is_trap: false,
			cost: 1.0,
		};
		for modifier in &elements[1..] {
			match modifier {
				Element::Earth => stats.is_trap = true,
				Element::Water => stats.range = 0.0,
				Element::Air => stats.range += 1.0,
				Element::Fire => stats.speed += 1.0,
				Element::Acid => stats.duration += 1.0,
				Element::Pressure => {
					stats.area *= 0.5;
					stats.strength += 0.5;
				}
				Element::Shock => {
					stats.duration *= 0.5;
					stats.strength *= 2.0;
				}
				Element::Radiance => stats.area += 1.0,
				Element::Life => stats.strength += 1.0,
				Element::Void => stats.is_inverted = !stats.is_inverted,
			}
			stats.cost += match modifier {
				Element::Earth => !stats.is_trap as u8 as f32,
				Element::Void => 0.0,
				Element::Life => stats.cost,
				Element::Water
				| Element::Air
				| Element::Fire
				| Element::Acid
				| Element::Shock
				| Element::Pressure
				| Element::Radiance => 1.0,
			}
		}
		Some(stats)
	}
}

impl Renderable<Color> for Spell {
	fn shader(&self) -> wgpu::ShaderModuleDescriptor {
		wgpu::include_wgsl!("flat.wgsl")
	}

	fn vertex_layout(&self) -> Vec<wgpu::VertexAttribute> {
		wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3].to_vec()
	}

	fn vertices(&self) -> Vec<Vertex<Color>> {
		let p = self.object.pos;
		let c = match self.element {
			Element::Earth => [0.4, 0.2, 0.0],
			Element::Void => [0.0, 0.0, 0.0],
			Element::Life => [1.0, 1.0, 1.0],
			Element::Water => [0.0, 0.0, 1.0],
			Element::Air => [0.0, 1.0, 1.0],
			Element::Fire => [1.0, 0.0, 0.0],
			Element::Acid => [0.0, 1.0, 0.0],
			Element::Shock => [1.0, 1.0, 0.0],
			Element::Pressure => [0.4, 0.6, 1.0],
			Element::Radiance => [1.0, 0.0, 1.0],
		};
		match self.object.shape {
			Shape::Aabb(Vector { x: w, y: h }) => vec![
				Vertex::new(p.x - w / 2.0, p.y - h / 2.0, c),
				Vertex::new(p.x + w / 2.0, p.y - h / 2.0, c),
				Vertex::new(p.x - w / 2.0, p.y + h / 2.0, c),
				Vertex::new(p.x + w / 2.0, p.y + h / 2.0, c),
				Vertex::new(p.x - w / 2.0, p.y + h / 2.0, c),
				Vertex::new(p.x + w / 2.0, p.y - h / 2.0, c),
			],
			Shape::Line(_dir) => todo!(),
		}
	}
}
