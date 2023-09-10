use std::ops::{Add, Sub, Mul, AddAssign, SubAssign, MulAssign};

pub struct Vector<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

	pub fn to<U>(self) -> Vector<U> where T: Into<U> {
		Vector {
			x: self.x.into(),
			y: self.y.into(),
		}
	}
}

impl<T> From<(T, T)> for Vector<T> {
	fn from((x, y): (T, T)) -> Self {
		Self { x, y }
	}
}

impl<T> Add for Vector<T> where T: Add<Output = T> {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x, 
			y: self.y + rhs.y,
		}
	}
}

impl<T> AddAssign for Vector<T> where T: AddAssign {
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
	}
}

impl<T> Sub for Vector<T> where T: Sub<Output = T> {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x - rhs.x, 
			y: self.y - rhs.y,
		}
	}
}

impl<T> SubAssign for Vector<T> where T: SubAssign {
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
	}
}

impl<T> Mul<T> for Vector<T> where T: Mul<Output = T> + Copy {
	type Output = Self;

	fn mul(self, rhs: T) -> Self::Output {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
		}
	}
}

impl<T> Mul<Vector<T>> for Vector<T> where T: Mul<Output = T> {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x * rhs.x, 
			y: self.y * rhs.y,
		}
	}
}

impl<T> MulAssign<T> for Vector<T> where T: MulAssign + Copy {
	fn mul_assign(&mut self, rhs: T) {
		self.x *= rhs;
		self.y *= rhs;
	}
}

impl<T> MulAssign<Vector<T>> for Vector<T> where T: MulAssign {
	fn mul_assign(&mut self, rhs: Self) {
		self.x *= rhs.x;
		self.y *= rhs.y;
	}
}
