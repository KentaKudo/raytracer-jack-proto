#[derive(Debug, PartialEq)]
pub struct Int32 {
    pub parts: [i16; 4],
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
        if other.parts == [0; 4] {
            panic!("divide by zero");
        }

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

        // let mut check = Self { parts: self.parts };
        // check.sub(&other);
        // if check.parts[3] >= 0x80 {
        //     *self = Self::from(0);
        //     return;
        // }

        // let mut q = Self::from(0);
        // if other.parts[3] == 0x00 && other.parts[2] < 0x80 {
        //     q = Self { parts: self.parts };
        //     q.div(&Self { parts: [0, other.parts[0], other.parts[1], other.parts[2]] });
        // };

        // let mut tmp = Self { parts: q.parts };
        // tmp.mul(&Self { parts: [0, other.parts[0], other.parts[1], other.parts[2]] });
        // self.sub(&tmp);

        // let mut quotient = 0;
        // loop {
        //     self.sub(&other);
        //     if self.parts[3] >= 0x80 {
        //         break
        //     }

        //     quotient += 1;
        // }

        // let mut r = Self { parts: [quotient, q.parts[0], q.parts[1], q.parts[2]] };
        // if negative {
        //     r.neg();
        // }

        let q = div_u4([
            self.parts[0] & 0x0F,
            self.parts[0] / 16,
            self.parts[1] & 0x0F,
            self.parts[1] / 16,
            self.parts[2] & 0x0F,
            self.parts[2] / 16,
            self.parts[3] & 0x0F,
            self.parts[3] / 16,
            0, 0, 0, 0, 0, 0, 0, 0,
        ], [
            other.parts[0] & 0x0F,
            other.parts[0] / 16,
            other.parts[1] & 0x0F,
            other.parts[1] / 16,
            other.parts[2] & 0x0F,
            other.parts[2] / 16,
            other.parts[3] & 0x0F,
            other.parts[3] / 16,
        ]);

        self.parts = [
            q[0] + q[1] * 16,
            q[2] + q[3] * 16,
            q[4] + q[5] * 16,
            q[6] + q[7] * 16,
        ];

        if negative {
            self.neg();
        }
    }
}

fn shift_arith_right(n: i16, sft: i16) -> i16 {
    let mut n = n;
    for _ in 0..sft {
        let neg = n < 0;
        if neg {
            n &= 0b0111111111111111;
        }

        n /= 2;

        if neg {
            n |= i16::MIN + 0b0100000000000000;
        }
    }
    n
}

fn shift_arith_left(n: i16, sft: i16) -> i16 {
    let mut n = n;
    for _ in 0..sft {
        n *= 2;
    }
    n
}

fn nlz(n: i16) -> i16 {
    match n {
        0 => 16,
        1 => 16 - 1,
        n if (2..4).contains(&n) => 16 - 2,
        n if (4..8).contains(&n) => 16 - 3,
        n if (8..16).contains(&n) => 16 - 4,
        n if (16..32).contains(&n) => 16 - 5,
        n if (32..64).contains(&n) => 16 - 6,
        n if (64..128).contains(&n) => 16 - 7,
        n if (128..256).contains(&n) => 16 - 8,
        n if (256..512).contains(&n) => 16 - 9,
        n if (512..1024).contains(&n) => 16 - 10,
        n if (1024..2048).contains(&n) => 16 - 11,
        n if (2048..4096).contains(&n) => 16 - 12,
        n if (4096..8192).contains(&n) => 16 - 13,
        n if (8192..16384).contains(&n) => 16 - 14,
        n if n < 0 => 16 - 15,
        _ => 0,
    }
}

pub fn div_u4(u: [i16; 16], v: [i16; 8]) -> [i16; 16] {
    if v == [0; 8] {
        panic!()
    }

    let base = 16;
    let mut n = 8;
    for i in v.iter().rev() {
        if *i == 0 {
            n -= 1;
        } else {
            break;
        }
    }

    let mut q = [0; 16];

    if n == 1 {
        let mut k = 0;
        for j in (0..16).rev() {
            q[j] = (k * base + u[j]) / v[0];
            k = (k * base + u[j]) - q[j] * v[0];
        }

        return q;
    }

    let s = nlz(v[n-1]) - 12;

    let mut vn = [0;8];
    for i in (1..n).rev() {
        vn[i] = (shift_arith_left(v[i], s) | shift_arith_right(v[i-1], 4-s)) & 0x0F;
    }
    vn[0] = shift_arith_left(v[0], s) & 0x0F;

    let mut un = [0;17];
    un[16] = shift_arith_right(u[15], 4 - s);
    for i in (1..u.len()).rev() {
        un[i] = (shift_arith_left(u[i], s) | shift_arith_right(u[i-1], 4-s)) & 0x0F;
    }
    un[0] = shift_arith_left(u[0], s) & 0x0F;

    for j in (0..=16-n).rev() {
        let mut qhat = (un[j+n] * base + un[j+n-1]) / vn[n-1];
        let mut rhat = (un[j+n] * base + un[j+n-1]) - qhat * vn[n-1];

        while qhat >= base || qhat * vn[n-2] > base * rhat + un[j+n-2] {
            qhat -= 1;
            rhat += vn[n-1];
            if rhat >= base {
                break;
            }
        }

        // multiply and subtract
        let mut k = 0;
        for i in 0..n {
            let p = qhat * vn[i];
            let t = un[i+j] - k - (p & 0x0F);
            un[i+j] = t & 0x0F;
            k = shift_arith_right(p, 4) - shift_arith_right(t, 4);
        }
        
        let t = un[j+n] - k;
        un[j+n] = t;

        q[j] = qhat;
        if t < 0 {
            q[j] -= 1;
            k = 0;
            for i in 0..n {
                let t = un[i+j] + vn[i] + k;
                un[i+j] = t & 0x0F;
                k = shift_arith_right(t, 4);
            }
            un[j+n] += k;
        }
    }

    q
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
    use rand::Rng;

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
    fn div_u4_vsize1() {
        for _ in 0..100 {
            let (a, b) = (
                rand::thread_rng().gen_range(0..i32::MAX),
                rand::thread_rng().gen_range(1..16)
            );

            let got = div_u4([
                ((a >> 0) & 0x0F) as i16,
                ((a >> 4) & 0x0F) as i16,
                ((a >> 8) & 0x0F) as i16,
                ((a >> 12) & 0x0F) as i16,
                ((a >> 16) & 0x0F) as i16,
                ((a >> 20) & 0x0F) as i16,
                ((a >> 24) & 0x0F) as i16,
                ((a >> 28) & 0x0F) as i16,
                0, 0, 0, 0, 0, 0, 0, 0,
            ], [b, 0, 0, 0, 0, 0, 0, 0]);

            let want = [
                (((a / b as i32) >> 0) & 0x0F) as i16,
                (((a / b as i32) >> 4) & 0x0F) as i16,
                (((a / b as i32) >> 8) & 0x0F) as i16,
                (((a / b as i32) >> 12) & 0x0F) as i16,
                (((a / b as i32) >> 16) & 0x0F) as i16,
                (((a / b as i32) >> 20) & 0x0F) as i16,
                (((a / b as i32) >> 24) & 0x0F) as i16,
                (((a / b as i32) >> 28) & 0x0F) as i16,
                0, 0, 0, 0, 0, 0, 0, 0,
            ];
            
            assert_eq!(got, want, "divide: {} / {}", a, b);
        }
    }

    #[test]
    fn div_u4_vsizen() {
        for _ in 0..100 {
            let (a, b) = (
                rand::thread_rng().gen_range(0..i32::MAX),
                rand::thread_rng().gen_range(1..i32::MAX)
            );

            let got = div_u4([
                ((a >> 0) & 0x0F) as i16,
                ((a >> 4) & 0x0F) as i16,
                ((a >> 8) & 0x0F) as i16,
                ((a >> 12) & 0x0F) as i16,
                ((a >> 16) & 0x0F) as i16,
                ((a >> 20) & 0x0F) as i16,
                ((a >> 24) & 0x0F) as i16,
                ((a >> 28) & 0x0F) as i16,
                0, 0, 0, 0, 0, 0, 0, 0,
            ], [
                ((b >> 0) & 0x0F) as i16,
                ((b >> 4) & 0x0F) as i16,
                ((b >> 8) & 0x0F) as i16,
                ((b >> 12) & 0x0F) as i16,
                ((b >> 16) & 0x0F) as i16,
                ((b >> 20) & 0x0F) as i16,
                ((b >> 24) & 0x0F) as i16,
                ((b >> 28) & 0x0F) as i16,
            ]);

            let want = [
                (((a / b as i32) >> 0) & 0x0F) as i16,
                (((a / b as i32) >> 4) & 0x0F) as i16,
                (((a / b as i32) >> 8) & 0x0F) as i16,
                (((a / b as i32) >> 12) & 0x0F) as i16,
                (((a / b as i32) >> 16) & 0x0F) as i16,
                (((a / b as i32) >> 20) & 0x0F) as i16,
                (((a / b as i32) >> 24) & 0x0F) as i16,
                (((a / b as i32) >> 28) & 0x0F) as i16,
                0, 0, 0, 0, 0, 0, 0, 0,
            ];
            
            assert_eq!(got, want, "divide: {} / {}", a, b);
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
