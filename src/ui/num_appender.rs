use std::iter::*;

pub trait NumAppender {
    fn append_real(&mut self, precision: usize, r: f32);
    fn append_int(&mut self, base: usize, leading_zeros: usize, i: i32); // base < 16
}

fn is_digit(ch: char) -> bool {
    let c = ch as usize;
    c >= '0' as usize && c <= '9' as usize
}

fn digit_to_num(ch: char) -> i32 {
    ch as i32 - '0' as i32
}

fn int_parser(s: &str) -> Result<i32, ()> {
    let mut v: i32 = 0;
    for c in s.chars() {
        match c {
            c if is_digit(c) => v = v * 10 + digit_to_num(c),
            _ => return Result::Err(()),
        }
    }
    Result::Ok(v)
}

fn parse_decimal(s: &str) -> Result<f64, ()> {
    let mut sign = 1.0;
    let mut p = s.chars().peekable();
    match p.peek() {
        Some('+') => {
            p.next();
        }
        Some('-') => {
            p.next();
            sign = -1.0
        }
        Some(c) if is_digit(*c) => (),
        _ => return Err(()),
    }

    // consume the leading zeros
    'zeros: loop {
        match p.peek() {
            Some('0') => {
                p.next();
            }
            _ => break 'zeros,
        }
    }

    let mut int_part = 0;

    'int_part: loop {
        match p.next() {
            Some(c) if is_digit(c) => int_part = int_part * 10 + digit_to_num(c),
            Some('.') => break 'int_part,
            None => return Ok((int_part as f64) * sign),
            _ => return Err(()),
        }
    }

    let mut decimal_part = 0.0;
    let mut power = 10.0;
    loop {
        match p.next() {
            Some(c) if is_digit(c) => {
                decimal_part += digit_to_num(c) as f64 / power;
                power *= 10.0;
            }

            None => return Ok((int_part as f64) * sign + decimal_part),
            _ => return Err(()),
        }
    }
}

const DIGITS: [char; 32] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V',
];

// base should be <= 32
fn print_int(base: usize, leading_zeros: usize, i: i32) -> String {
    assert!(base <= 32);
    let mut int_str = Vec::new();
    let orig_i = i;
    let mut i = (i.signum() * i) as usize;
    while i != 0 {
        int_str.push(DIGITS[i % base]);
        i /= base;
    }

    for _ in int_str.len()..leading_zeros {
        int_str.push('0');
    }

    if orig_i < 0 {
        int_str.push('-')
    }

    int_str.reverse();
    String::from_iter(int_str)
}

fn print_float(f: f64, p: usize) -> String {
    let mut int_str = Vec::new();
    let mut i = (f.signum() * f) as usize;
    while i != 0 {
        int_str.push(DIGITS[i % 10]);
        i /= 10;
    }

    if int_str.len() == 0 {
        int_str.push('0');
    }

    if f < 0.0 {
        int_str.push('-')
    }

    int_str.reverse();

    let mut pow_10 = 1;
    for _ in 0..p {
        pow_10 *= 10;
    }

    // i2 = i * 10^p
    // d2 = f * 10^p
    // i = 10^p + (d2 - i2) <- needed so we can get the number of leading 0 right
    let int2_part = ((f.signum() * f) as usize) * pow_10;
    let mut dec2_part = pow_10 + ((f.signum() * f) * (pow_10 as f64)) as usize - int2_part;

    if dec2_part == pow_10 || p == 0 {
        return String::from_iter(int_str);
    }

    int_str.push('.');

    let mut dec_str = Vec::new();
    for _ in 0..p {
        dec_str.push(DIGITS[dec2_part % 10]);
        dec2_part /= 10;
    }
    dec_str.reverse();

    int_str.append(&mut dec_str);
    String::from_iter(int_str)
}

impl NumAppender for String {
    fn append_real(&mut self, precision: usize, v: f32) {
        let s = print_float(v as f64, precision);
        self.push_str(s.as_str());
    }

    fn append_int(&mut self, base: usize, leading_zeros: usize, v: i32) {
        let s = print_int(base, leading_zeros, v);
        self.push_str(s.as_str());
    }
}

pub trait FromDecimal {
    fn from_decimal(s: &str) -> Result<Self, ()>
    where
        Self: Sized;
}

impl FromDecimal for f64 {
    fn from_decimal(s: &str) -> Result<Self, ()>
    where
        Self: Sized,
    {
        parse_decimal(s)
    }
}

impl FromDecimal for f32 {
    fn from_decimal(s: &str) -> Result<Self, ()>
    where
        Self: Sized,
    {
        parse_decimal(s).map(|x| x as _)
    }
}
