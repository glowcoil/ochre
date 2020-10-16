use std::ops;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[inline]
    pub fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x: x, y: y }
    }

    #[inline]
    pub fn dot(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn cross(self, other: Vec2) -> f32 {
        self.x * other.y - self.y * other.x
    }

    #[inline]
    pub fn distance(self, other: Vec2) -> f32 {
        (other - self).length()
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    #[inline]
    pub fn normalized(self) -> Vec2 {
        (1.0 / self.length()) * self
    }

    #[inline]
    pub fn lerp(t: f32, a: Vec2, b: Vec2) -> Vec2 {
        (1.0 - t) * a + t * b
    }

    #[inline]
    pub fn min(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    #[inline]
    pub fn max(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}

impl ops::Add for Vec2 {
    type Output = Vec2;
    #[inline]
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, other: Vec2) {
        *self = *self + other;
    }
}

impl ops::Sub for Vec2 {
    type Output = Vec2;
    #[inline]
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, other: Vec2) {
        *self = *self - other;
    }
}

impl ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::Mul<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl ops::MulAssign<f32> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, other: f32) {
        *self = *self * other;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Mat2x2(pub [f32; 4]);

impl Mat2x2 {
    /* row-major order */
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Mat2x2 {
        Mat2x2([a, b, c, d])
    }

    pub fn id() -> Mat2x2 {
        Mat2x2([1.0, 0.0, 0.0, 1.0])
    }

    pub fn scale(scale: f32) -> Mat2x2 {
        Mat2x2([scale, 0.0, 0.0, scale])
    }

    pub fn rotate(angle: f32) -> Mat2x2 {
        Mat2x2([angle.cos(), angle.sin(), -angle.sin(), angle.cos()])
    }
}

impl ops::Mul<Mat2x2> for Mat2x2 {
    type Output = Mat2x2;
    #[inline]
    fn mul(self, rhs: Mat2x2) -> Mat2x2 {
        Mat2x2([
            self.0[0] * rhs.0[0] + self.0[1] * rhs.0[2],
            self.0[0] * rhs.0[1] + self.0[1] * rhs.0[3],
            self.0[2] * rhs.0[0] + self.0[3] * rhs.0[2],
            self.0[2] * rhs.0[1] + self.0[3] * rhs.0[3],
        ])
    }
}

impl ops::Mul<Vec2> for Mat2x2 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.0[0] * rhs.x + self.0[1] * rhs.y,
            y: self.0[2] * rhs.x + self.0[3] * rhs.y,
        }
    }
}

impl ops::MulAssign<Mat2x2> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, other: Mat2x2) {
        *self = other * *self;
    }
}

impl ops::Mul<Mat2x2> for f32 {
    type Output = Mat2x2;
    #[inline]
    fn mul(self, rhs: Mat2x2) -> Mat2x2 {
        Mat2x2([
            self * rhs.0[0],
            self * rhs.0[1],
            self * rhs.0[2],
            self * rhs.0[3],
        ])
    }
}

impl ops::Mul<f32> for Mat2x2 {
    type Output = Mat2x2;
    #[inline]
    fn mul(self, rhs: f32) -> Mat2x2 {
        rhs * self
    }
}

impl ops::MulAssign<f32> for Mat2x2 {
    #[inline]
    fn mul_assign(&mut self, other: f32) {
        *self = *self * other;
    }
}
