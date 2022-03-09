use std::os::unix::thread::RawPthread;

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
    pub fn sub(&mut self, other: &mut Self) {
        other.neg();
        self.add(&other);
    }
}

impl Int32 {
    pub fn mul(&mut self, other: &mut Self) {
        let negative = (self.parts[3] >= 0x80) ^ (other.parts[3] >= 0x80);

        if self.parts[3] >= 0x80 {
            self.neg();
        }

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
    }

    #[test]
    fn sub() {
        let mut n = Int32::from(1);
        n.sub(&mut Int32::from(1));
        assert_eq!(n, 0.into());

        n = Int32::from(0x09999999);
        n.sub(&mut Int32::from(0x06666666));
        assert_eq!(n, 0x03333333.into());

        n = Int32::from(0x100);
        n.sub(&mut Int32::from(1));
        assert_eq!(n, 0xFF.into());

        n = Int32::from(0x10000);
        n.sub(&mut Int32::from(1));
        assert_eq!(n, 0xFFFF.into());

        n = Int32::from(0x1000000);
        n.sub(&mut Int32::from(1));
        assert_eq!(n, 0xFFFFFF.into());

        n = Int32::from(0);
        n.sub(&mut Int32::from(1));
        assert_eq!(n, (-1).into());
    }

    #[test]
    fn mul() {
        let mut n = Int32::from(68375);
        n.mul(&mut Int32::from(2317));
        assert_eq!(n, 158424875.into());

        let mut n = Int32::from(68375);
        n.mul(&mut Int32::from(0));
        assert_eq!(n, 0.into());

        let mut n = Int32::from(68375);
        n.mul(&mut Int32::from(1));
        assert_eq!(n, 68375.into());

        let mut n = Int32::from(1);
        n.mul(&mut Int32::from(2317));
        assert_eq!(n, 2317.into());

        let mut n = Int32::from(68375);
        n.mul(&mut Int32::from(-2317));
        assert_eq!(n, (-158424875).into());

        let mut n = Int32::from(-68375);
        n.mul(&mut Int32::from(2317));
        assert_eq!(n, (-158424875).into());

        let mut n = Int32::from(-68375);
        n.mul(&mut Int32::from(-2317));
        assert_eq!(n, 158424875.into());

        let mut n = Int32::from(0xFF);
        n.mul(&mut Int32::from(0xFF));
        assert_eq!(n, (0xFF * 0xFF).into());

        let mut n = Int32::from(-0xFF);
        n.mul(&mut Int32::from(-0xFF));
        assert_eq!(n, (0xFF * 0xFF).into());
    }
}
