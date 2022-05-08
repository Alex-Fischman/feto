use std::ops::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vector {
	pub x: f32,
	pub y: f32,
}

impl Add for Vector {
	type Output = Vector;
	fn add(self, other: Vector) -> Vector {
		Vector { x: self.x + other.x, y: self.y + other.y }
	}
}

impl Sub for Vector {
	type Output = Vector;
	fn sub(self, other: Vector) -> Vector {
		Vector { x: self.x - other.x, y: self.y - other.y }
	}
}

impl Mul for Vector {
	type Output = Vector;
	fn mul(self, other: Vector) -> Vector {
		Vector { x: self.x * other.x, y: self.y * other.y }
	}
}

impl Mul<f32> for Vector {
	type Output = Vector;
	fn mul(self, other: f32) -> Vector {
		Vector { x: self.x * other, y: self.y * other }
	}
}

impl AddAssign for Vector {
	fn add_assign(&mut self, other: Vector) {
		*self = *self + other;
	}
}

impl MulAssign<f32> for Vector {
	fn mul_assign(&mut self, other: f32) {
		*self = *self * other;
	}
}

#[allow(dead_code)]
impl Vector {
	pub fn new(x: f32, y: f32) -> Vector {
		Vector { x, y }
	}

	pub fn dot(self, other: Vector) -> f32 {
		self.x * other.x + self.y * other.y
	}

	pub fn length(self) -> f32 {
		self.dot(self).sqrt()
	}

	pub fn normalized(self) -> Option<Vector> {
		let l = self.length();
		if l == 0.0 {
			None
		} else {
			Some(self * (1.0 / l))
		}
	}
}
