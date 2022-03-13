use lazy_static::lazy_static;
use crate::int32::{Int32, div_u4};

lazy_static! {
    static ref SCALE_FACTOR: Int32 = Int32::from(0xFFFF);
}

#[derive(Debug, PartialEq)]
pub struct Number(Int32);

impl From<i16> for Number {
    fn from(n: i16) -> Self {
        let i = Int32::from(n as i32);
        Self(Int32 { parts: [0, 0, i.parts[0], i.parts[1]] })
    }
}

impl Number {
    pub fn add(&mut self, other: &Self) {
        self.0.add(&other.0);
    }

    pub fn sub(&mut self, other: &Self) {
        self.0.sub(&other.0);
    }
}

impl Number {
    pub fn mul(&mut self, other: &Self) {
        let mut decdec = Int32 {
            parts: if self.0.parts[3] < 0x80 {
                [self.0.parts[2], self.0.parts[3], 0, 0]
            } else {
                [self.0.parts[2], self.0.parts[3], 0xFF, 0xFF]
            }
        };
        decdec.mul(&Int32 {
            parts: if other.0.parts[3] < 0x80 {
                [other.0.parts[2], other.0.parts[3], 0, 0]
            } else {
                [other.0.parts[2], other.0.parts[3], 0xFF, 0xFF]
            }
        });

        let mut decfrac = Int32 {
            parts: if self.0.parts[3] < 0x80 {
                [self.0.parts[2], self.0.parts[3], 0, 0]
            } else {
                [self.0.parts[2], self.0.parts[3], 0xFF, 0xFF]
            }
        };
        decfrac.mul(&Int32 { parts: [other.0.parts[0], other.0.parts[1], 0, 0] });

        let mut fracdec = Int32 {parts: [self.0.parts[0], self.0.parts[1], 0, 0] };
        fracdec.mul(&Int32 {
            parts: if other.0.parts[3] < 0x80 {
                [other.0.parts[2], other.0.parts[3], 0, 0]
            } else {
                [other.0.parts[2], other.0.parts[3], 0xFF, 0xFF]
            }
        });

        let mut fracfrac = Int32 { parts: [self.0.parts[0], self.0.parts[1], 0, 0] };
        fracfrac.mul(&Int32 { parts: [other.0.parts[0], other.0.parts[1], 0, 0] });

        let mut r = Int32 { parts: [fracfrac.parts[2], fracfrac.parts[3], 0, 0] };
        r.add(&decfrac);
        r.add(&fracdec);
        r.add(&Int32 { parts: [0, 0, decdec.parts[0], decdec.parts[1]] });

        self.0.parts = r.parts;
    }
}

impl Number {
    pub fn div(&mut self, other: &Self) {
        if other.0.parts == [0; 4] {
            panic!("divide by zero");
        }

        let negative = (self.0.parts[3] >= 0x80) ^ (other.0.parts[3] >= 0x80);

        if self.0.parts[3] >= 0x80 {
            self.0.neg();
        }

        let mut other = Self(Int32 { parts: other.0.parts });
        if other.0.parts[3] >= 0x80 {
            other.0.neg();
        }

        let q = div_u4([
            0, 0, 0, 0,
            self.0.parts[0] & 0x0F,
            self.0.parts[0] / 16,
            self.0.parts[1] & 0x0F,
            self.0.parts[1] / 16,
            self.0.parts[2] & 0x0F,
            self.0.parts[2] / 16,
            self.0.parts[3] & 0x0F,
            self.0.parts[3] / 16,
            0, 0, 0, 0,
        ], [
            other.0.parts[0] & 0x0F,
            other.0.parts[0] / 16,
            other.0.parts[1] & 0x0F,
            other.0.parts[1] / 16,
            other.0.parts[2] & 0x0F,
            other.0.parts[2] / 16,
            other.0.parts[3] & 0x0F,
            other.0.parts[3] / 16,
        ]);

        self.0.parts = [
            q[0] + q[1] * 16,
            q[2] + q[3] * 16,
            q[4] + q[5] * 16,
            q[6] + q[7] * 16,
        ];

        if negative {
            self.0.neg();
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn add() {
        let mut n = Number::from(1);
        n.add(&Number::from(1));
        assert_eq!(n, 2.into());
    }

    #[test]
    fn mul() {
        let mut n = Number(Int32::from(131072));
        n.mul(&Number(Int32::from(16384)));
        assert_eq!(n, Number(Int32::from(32768)));

        let mut n = Number::from(2);
        n.mul(&Number::from(2));
        assert_eq!(n, Number::from(4));

        let mut n = Number(Int32::from(-1630363593));
        n.mul(&Number(Int32::from(702874066)));
        let want = ((-1630363593 as i64) * (702874066 as i64) >> 16) as i32;
        assert_eq!(n, Number(Int32::from(want)));

        for _ in 0..100 {
            let (a, b) = (rand::random::<i32>(), rand::random::<i32>());
            let mut n = Number(Int32::from(a));
            n.mul(&Number(Int32::from(b)));
            let want = ((a as i64).wrapping_mul(b as i64) >> 16) as i32;
            assert_eq!(n, Number(Int32::from(want)), "multiply: {} * {}", a, b);
        }
    }

    #[test]
    fn div() {
        let mut n = Number(Int32::from(131072));
        n.div(&Number(Int32::from(16384)));
        assert_eq!(n, Number(Int32::from(524288)));

        for _ in 0..100 {
            let (a, b) = (rand::random::<i32>(), rand::random::<i32>());
            let mut n = Number(Int32::from(a));
            n.div(&Number(Int32::from(b)));
            let want = (((a as i64) << 16) / (b as i64)) as i32;
            assert_eq!(n, Number(Int32::from(want)), "divide: {} / {}", a, b);
        }
    }
}
