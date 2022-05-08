use crate::vector::Vector;

pub struct Object {
	pub pos: Vector,
	pub vel: Vector,
	pub shape: Shape,
}

pub enum Shape {
	Aabb(Vector), // axis aligned bounding box: vector holds width and height
	Line(Vector), // line segment: vector is the offset of the endpoint
}

impl Object {
	pub fn new(x: f32, y: f32, shape: Shape) -> Object {
		Object { pos: Vector { x, y }, vel: Vector::new(0.0, 0.0), shape }
	}

	pub fn move_and_collide(&mut self, colliders: &[&Object], delta_time: f32) -> bool {
		let mut collided = false;
		self.pos += self.vel * delta_time;
		'outer: loop {
			for collider in colliders {
				if collide(self, collider) {
					collided = true;
					match (&self.shape, &collider.shape) {
						(
							Shape::Aabb(Vector { x: sw, y: sh }),
							Shape::Aabb(Vector { x: cw, y: ch }),
						) => {
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
						_ => todo!(),
					}
					continue 'outer;
				}
			}
			break;
		}
		collided
	}
}

fn aabb_aabb(amin: Vector, amax: Vector, bmin: Vector, bmax: Vector) -> bool {
	amin.x < bmax.x && amax.x > bmin.x && amin.y < bmax.y && amax.y > bmin.y
}

fn point_aabb(p: Vector, min: Vector, max: Vector) -> bool {
	min.x < p.x && min.y < p.y && max.x > p.x && max.y > p.y
}

fn line_aabb(pos: Vector, dir: Vector, min: Vector, max: Vector) -> bool {
	let t = dir.length();
	let d = match dir.normalized() {
		Some(v) => v,
		None => return point_aabb(pos, min, max),
	};
	let t0 = (min - pos) * Vector::new(1.0 / d.x, 1.0 / d.y);
	let t1 = (max - pos) * Vector::new(1.0 / d.x, 1.0 / d.y);
	let tmin = f32::max(f32::min(t0.x, t1.x), f32::min(t0.y, t1.y));
	let tmax = f32::min(f32::max(t0.x, t1.x), f32::max(t0.y, t1.y));
	return tmin <= t && t <= tmax;
}

pub fn collide(a: &Object, b: &Object) -> bool {
	match (&a.shape, &b.shape) {
		(Shape::Aabb(asize), Shape::Aabb(bsize)) => aabb_aabb(
			a.pos - *asize * 0.5,
			a.pos + *asize * 0.5,
			b.pos - *bsize * 0.5,
			b.pos + *bsize * 0.5,
		),
		(Shape::Line(dir), Shape::Aabb(size)) => {
			line_aabb(a.pos, *dir, b.pos - *size * 0.5, b.pos + *size * 0.5)
		}
		(Shape::Aabb(size), Shape::Line(dir)) => {
			line_aabb(b.pos, *dir, a.pos - *size * 0.5, a.pos + *size * 0.5)
		}
		_ => todo!(),
	}
}
