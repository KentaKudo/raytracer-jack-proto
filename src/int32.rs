#[derive(Debug, PartialEq)]
pub struct Int32 {
    parts: [i16; 4],
}

impl From<i32> for Int32 {
    fn from(n: i32) -> Self {
        Int32 {
            parts: [
                ((n >> 0) & 0xFF) as i16,
                ((n >> 8) & 0xFF) as i16,
                ((n >> 16) & 0xFF) as i16,
                ((n >> 24) & 0xFF) as i16,
            ],
        }
    }
}

impl From<Int32> for i32 {
    fn from(n: Int32) -> Self {
        (n.parts[3] as i32) << 24 | (n.parts[2] as i32) << 16 | (n.parts[1] as i32) << 8 | (n.parts[0] as i32) << 0
    }
}

impl Int32 {
    pub fn neg(&mut self) {
        self.parts[0] = !self.parts[0] & 0xFF;
        self.parts[1] = !self.parts[1] & 0xFF;
        self.parts[2] = !self.parts[2] & 0xFF;
        self.parts[3] = !self.parts[3] & 0xFF;
        self.parts[0] = self.parts[0] + 1;

        if self.parts[0] > 0xFF {
            self.parts[0] = 0;
            self.parts[1] += 1;
        }

        if self.parts[1] > 0xFF {
            self.parts[1] = 0;
            self.parts[2] += 1;
        }

        if self.parts[2] > 0xFF {
            self.parts[2] = 0;
            self.parts[3] += 1;
        }

        if self.parts[3] > 0xFF {
            self.parts[3] = 0;
        }
    }
}

impl Int32 {
    pub fn add(&mut self, other: &Self) {
        self.parts[0] += other.parts[0];
        self.parts[1] += other.parts[1];
        self.parts[2] += other.parts[2];
        self.parts[3] += other.parts[3];

        if self.parts[0] > 0xFF {
            self.parts[0] -= 0x100;
            self.parts[1] += 1;
        }

        if self.parts[1] > 0xFF {
            self.parts[1] -= 0x100;
            self.parts[2] += 1;
        }

        if self.parts[2] > 0xFF {
            self.parts[2] -= 0x100;
            self.parts[3] += 1;
        }

        if self.parts[3] > 0xFF {
            self.parts[3] -= 0x100;
        }
    }
}

impl Int32 {
    pub fn sub(&mut self, other: &Self) {
        let mut other = Self { parts: other.parts };
        other.neg();
        self.add(&other);
    }
}

impl Int32 {
    pub fn mul(&mut self, other: &Self) {
        let negative = (self.parts[3] >= 0x80) ^ (other.parts[3] >= 0x80);

        if self.parts[3] >= 0x80 {
            self.neg();
        }

        let mut other = Self { parts: other.parts };
        if other.parts[3] >= 0x80 {
            other.neg();
        }

        let self_chunked = [
            self.parts[0] & 0x0F,
            self.parts[0] / 16,
            self.parts[1] & 0x0F,
            self.parts[1] / 16,
            self.parts[2] & 0x0F,
            self.parts[2] / 16,
            self.parts[3] & 0x0F,
            self.parts[3] / 16,
        ];

        let other_chunked = [
            other.parts[0] & 0x0F,
            other.parts[0] / 16,
            other.parts[1] & 0x0F,
            other.parts[1] / 16,
            other.parts[2] & 0x0F,
            other.parts[2] / 16,
            other.parts[3] & 0x0F,
            other.parts[3] / 16,
        ];

        let mut mul = [0; 8];
        for (i, lhs) in self_chunked.iter().enumerate() {
            for (j, rhs) in other_chunked.iter().enumerate() {
                if i + j > 7 {
                    continue;
                }

                mul[i + j] += rhs * lhs;
            }
        }

        let mut r = Self::from(0);
        r.parts[0] = mul[0] + mul[1] * 16;
        if r.parts[0] > 0xFF {
            r.parts[1] = r.parts[0] / 256;
            r.parts[0] = r.parts[0] & 0xFF;
        }

        r.parts[1] += mul[2] + mul[3] * 16;
        if r.parts[1] > 0xFF {
            r.parts[2] = r.parts[1] / 256;
            r.parts[1] = r.parts[1] & 0xFF;
        }

        r.parts[2] += mul[4] + mul[5] * 16;
        if r.parts[2] > 0xFF {
            r.parts[3] = r.parts[2] / 256;
            r.parts[2] = r.parts[2] & 0xFF;
        }

        r.parts[3] += mul[6] + mul[7] * 16;
        if r.parts[3] > 0xFF {
            r.parts[3] = r.parts[3] & 0xFF;
        }

        if negative {
            r.neg();
        }

        *self = r;
    }
}

impl Int32 {
    pub fn div(&mut self, other: &Self) {
        let negative = (self.parts[3] >= 0x80) ^ (other.parts[3] >= 0x80);

        if self.parts[3] >= 0x80 {
            self.neg();
        }

        let mut other = Self { parts: other.parts };
        if other.parts[3] >= 0x80 {
            other.neg();
        }

        // let mut quotient = 0;
        // loop {
        //     if other.parts[3] > self.parts[3]
        //         || (other.parts[3] == self.parts[3] && other.parts[2] > self.parts[2])
        //         || (other.parts[3] == self.parts[3] && other.parts[2] == self.parts[2] && other.parts[1] > self.parts[1])
        //         || (other.parts[3] == self.parts[3] && other.parts[2] == self.parts[2] && other.parts[1] == self.parts[1] && other.parts[0] > self.parts[0]) {
        //         break;
        //     }

        //     self.sub(&other);
        //     quotient += 1;
        // }

        // *self = Int32::from(if negative { -quotient } else { quotient });

        let mut check = Self { parts: self.parts };
        check.sub(&other);
        if check.parts[3] >= 0x80 {
            *self = Self::from(0);
            return;
        }

        let mut q = Self::from(0);
        if other.parts[3] == 0x00 && other.parts[2] < 0x80 {
            q = Self { parts: self.parts };
            q.div(&Self { parts: [0, other.parts[0], other.parts[1], other.parts[2]] });
        };

        let mut tmp = Self { parts: q.parts };
        tmp.mul(&Self { parts: [0, other.parts[0], other.parts[1], other.parts[2]] });
        self.sub(&tmp);

        let mut quotient = 0;
        loop {
            self.sub(&other);
            if self.parts[3] >= 0x80 {
                break
            }

            quotient += 1;
        }

        let mut r = Self { parts: [quotient, q.parts[0], q.parts[1], q.parts[2]] };
        if negative {
            r.neg();
        }

        *self = r;
    }
}

impl Int32 {
    pub fn sqrt(&mut self) {
        if self.parts[3] >= 0x80 {
            panic!()
        }

        if self.parts == [0, 0, 0, 0] {
            *self = Self::from(0);
            return;
        }

        let mut guess = Self::from(5);
        for _ in 0..20 {
            let mut inv = Self { parts: self.parts };
            inv.div(&guess);

            guess.add(&inv);
            guess.div(&Self::from(2));
        }

        *self = guess;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from() {
        assert_eq!(
            Int32::from(1),
            Int32 {
                parts: [1, 0, 0, 0]
            }
        );

        assert_eq!(
            Int32::from(-1),
            Int32 {
                parts: [0xFF, 0xFF, 0xFF, 0xFF]
            }
        );
    }

    #[test]
    fn neg() {
        let mut n = Int32::from(1);
        n.neg();
        assert_eq!(n, (-1).into());

        n = Int32::from(0x00000100);
        n.neg();
        assert_eq!(n, (-0x00000100).into());

        n = Int32::from(0x00010000);
        n.neg();
        assert_eq!(n, (-0x00010000).into());

        n = Int32::from(0x01000000);
        n.neg();
        assert_eq!(n, (-0x01000000).into());

        n = Int32::from(0x00000000);
        n.neg();
        assert_eq!(n, (-0x00000000).into());

        n = Int32::from(0x06666666);
        n.neg();
        assert_eq!(n, (-0x06666666).into());

        let a = rand::random::<i32>();
        let mut n = Int32::from(a);
        n.neg();
        assert_eq!(n, (a.wrapping_neg()).into(), "negate: -{}", a);
    }

    #[test]
    fn add() {
        let mut n = Int32::from(1);
        n.add(&Int32::from(1));
        assert_eq!(n, 2.into());

        n = Int32::from(0x06996699);
        n.add(&Int32::from(0x09669966));
        assert_eq!(n, 0x0FFFFFFF.into());

        n = Int32::from(1);
        n.add(&Int32::from(0xFF));
        assert_eq!(n, 0x100.into());

        n = Int32::from(1);
        n.add(&Int32::from(0xFFFF));
        assert_eq!(n, 0x10000.into());

        n = Int32::from(1);
        n.add(&Int32::from(0xFFFFFF));
        assert_eq!(n, 0x1000000.into());

        n = Int32::from(-1);
        n.add(&Int32::from(1));
        assert_eq!(n, 0.into());

        n = Int32::from(0x09999999);
        n.add(&Int32::from(-0x06666666));
        assert_eq!(n, (0x03333333).into());

        let (a, b) = (rand::random::<i32>(), rand::random::<i32>());
        let mut n = Int32::from(a);
        n.add(&Int32::from(b));
        assert_eq!(n, (a.wrapping_add(b)).into(), "add: {} + {}", a, b);
    }

    #[test]
    fn sub() {
        let mut n = Int32::from(1);
        n.sub(&Int32::from(1));
        assert_eq!(n, 0.into());

        n = Int32::from(0x09999999);
        n.sub(&Int32::from(0x06666666));
        assert_eq!(n, 0x03333333.into());

        n = Int32::from(0x100);
        n.sub(&Int32::from(1));
        assert_eq!(n, 0xFF.into());

        n = Int32::from(0x10000);
        n.sub(&Int32::from(1));
        assert_eq!(n, 0xFFFF.into());

        n = Int32::from(0x1000000);
        n.sub(&Int32::from(1));
        assert_eq!(n, 0xFFFFFF.into());

        n = Int32::from(0);
        n.sub(&Int32::from(1));
        assert_eq!(n, (-1).into());

        let (a, b) = (rand::random::<i32>(), rand::random::<i32>());
        let mut n = Int32::from(a);
        n.sub(&Int32::from(b));
        assert_eq!(n, (a.wrapping_sub(b)).into(), "subtract: {} - {}", a, b);
    }

    #[test]
    fn mul() {
        let mut n = Int32::from(68375);
        n.mul(&Int32::from(2317));
        assert_eq!(n, 158424875.into());

        let mut n = Int32::from(68375);
        n.mul(&Int32::from(0));
        assert_eq!(n, 0.into());

        let mut n = Int32::from(68375);
        n.mul(&Int32::from(1));
        assert_eq!(n, 68375.into());

        let mut n = Int32::from(1);
        n.mul(&Int32::from(2317));
        assert_eq!(n, 2317.into());

        let mut n = Int32::from(68375);
        n.mul(&Int32::from(-2317));
        assert_eq!(n, (-158424875).into());

        let mut n = Int32::from(-68375);
        n.mul(&Int32::from(2317));
        assert_eq!(n, (-158424875).into());

        let mut n = Int32::from(-68375);
        n.mul(&Int32::from(-2317));
        assert_eq!(n, 158424875.into());

        let mut n = Int32::from(0xFF);
        n.mul(&Int32::from(0xFF));
        assert_eq!(n, (0xFF * 0xFF).into());

        let mut n = Int32::from(-0xFF);
        n.mul(&Int32::from(-0xFF));
        assert_eq!(n, (0xFF * 0xFF).into());

        for _ in 0..100 {
            let (a, b) = (rand::random::<i32>(), rand::random::<i32>());
            let mut n = Int32::from(a);
            n.mul(&Int32::from(b));
            assert_eq!(n, (a.wrapping_mul(b)).into(), "multiply: {} * {}", a, b);
        }
    }

    #[test]
    fn div() {
        let mut n = Int32::from(781);
        n.div(&Int32::from(330519));
        assert_eq!(n, 0.into());

        let mut n = Int32::from(330519);
        n.div(&Int32::from(781));
        assert_eq!(n, 423.into());

        let mut n = Int32::from(1);
        n.div(&Int32::from(1));
        assert_eq!(n, 1.into());

        let mut n = Int32::from(-1);
        n.div(&Int32::from(1));
        assert_eq!(n, (-1).into());

        let mut n = Int32::from(1);
        n.div(&Int32::from(-1));
        assert_eq!(n, (-1).into());

        let mut n = Int32::from(256);
        n.div(&Int32::from(3));
        assert_eq!(n, 85.into());

        let mut n = Int32::from(1543938581);
        n.div(&Int32::from(-623681255));
        assert_eq!(n, (1543938581 / -623681255).into());

        let mut n = Int32::from(884474092);
        n.div(&Int32::from(13586197));
        assert_eq!(n, (884474092 / 13586197).into());

        for _ in 0..100 {
            let (a, b) = (rand::random::<i32>(), rand::random::<i32>());
            let mut n = Int32::from(a);
            n.div(&Int32::from(b));
            assert_eq!(n, (a / b).into(), "divide: {} / {}", a, b);
        }
    }

    #[test]
    fn sqrt() {
        let mut n = Int32::from(1);
        n.sqrt();
        assert_eq!(n, 1.into());

        let mut n = Int32::from(4);
        n.sqrt();
        assert_eq!(n, 2.into());

        for _ in 0..100 {
            let a = rand::random::<i32>() & 0x7FFFFFFF;
            let mut n = Int32::from(a);
            n.sqrt();
            assert_eq!(n, ((a as f64).sqrt() as i32).into(), "sqrt: {}", a);
        }
    }
}
