use crate::render::Vertex;
use crate::vector::Vector;

type Color = [f32; 3];

pub struct Object {
	pub pos: Vector,
	pub vel: Vector,
	color: Color,
	shape: Shape,
}

pub enum Shape {
	Aabb(f32, f32),
}

impl Object {
	pub fn new(x: f32, y: f32, color: Color, shape: Shape) -> Object {
		Object { pos: Vector { x, y }, vel: Vector::new(0.0, 0.0), color, shape }
	}

	pub fn move_and_collide(&mut self, colliders: &[Object], delta_time: f32) -> bool {
		let mut collided = false;
		self.pos += self.vel * delta_time;
		'outer: loop {
			for collider in colliders {
				if collide(self, collider) {
					collided = true;
					match (&self.shape, &collider.shape) {
						(Shape::Aabb(sw, sh), Shape::Aabb(cw, ch)) => {
							let red =
								((self.pos.y - sh / 2.0) - (collider.pos.y + ch / 2.0)).abs();
							let black =
								((self.pos.x - sw / 2.0) - (collider.pos.x + cw / 2.0)).abs();
							let yellow =
								((self.pos.y + sh / 2.0) - (collider.pos.y - ch / 2.0)).abs();
							let purple =
								((self.pos.x + sw / 2.0) - (collider.pos.x - cw / 2.0)).abs();
							self.pos += if red < black && red < yellow && red < purple {
								Vector::new(0.0, red)
							} else if black < red && black < yellow && black < purple {
								Vector::new(black, 0.0)
							} else if yellow < red && yellow < black && yellow < purple {
								Vector::new(0.0, -yellow)
							} else {
								Vector::new(-purple, 0.0)
							};
						}
					}
					continue 'outer;
				}
			}
			break;
		}
		collided
	}
}

pub fn collide(a: &Object, b: &Object) -> bool {
	match (&a.shape, &b.shape) {
		(Shape::Aabb(aw, ah), Shape::Aabb(bw, bh)) => {
			a.pos.x - aw / 2.0 < b.pos.x + bw / 2.0
				&& a.pos.x + aw / 2.0 > b.pos.x - bw / 2.0
				&& a.pos.y - ah / 2.0 < b.pos.y + bh / 2.0
				&& a.pos.y + ah / 2.0 > b.pos.y - bh / 2.0
		}
	}
}

impl crate::render::Renderable<Color> for Object {
	fn shader(&self) -> wgpu::ShaderModuleDescriptor {
		wgpu::include_wgsl!("flat.wgsl")
	}

	fn vertex_layout(&self) -> Vec<wgpu::VertexAttribute> {
		wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3].to_vec()
	}

	fn vertices(&self) -> Vec<Vertex<Color>> {
		match self.shape {
			Shape::Aabb(w, h) => vec![
				Vertex::new(self.pos.x - w / 2.0, self.pos.y - h / 2.0, self.color),
				Vertex::new(self.pos.x + w / 2.0, self.pos.y - h / 2.0, self.color),
				Vertex::new(self.pos.x - w / 2.0, self.pos.y + h / 2.0, self.color),
				Vertex::new(self.pos.x + w / 2.0, self.pos.y + h / 2.0, self.color),
				Vertex::new(self.pos.x - w / 2.0, self.pos.y + h / 2.0, self.color),
				Vertex::new(self.pos.x + w / 2.0, self.pos.y - h / 2.0, self.color),
			],
		}
	}
}
