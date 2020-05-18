// a lazy IEEE 754 extended precision number parser
// https://en.wikipedia.org/wiki/Extended_precision#x86_extended_precision_format
use std::ops::Mul;

const EXP_BIAS: i16 = 16383;
const FIRST_POS: u8 = 0b10000000;

// when offset 0 && bias 0, MSB = x * 2^0
fn read_binary_fraction(byte: u8, byte_offset: u32, bias: u8) -> f64 {
    let mut res = 0f64;
    for idx in 0..8 {
        if ((byte << idx) & FIRST_POS) == FIRST_POS {
            let power: i32 =
                ((idx + bias as i32) + (8 * byte_offset) as i32) * -1;
            res += 2f64.powi(power);
        }
    }
    res
}

// TOOD return err
pub fn parse_extended_precision_bytes(b: [u8; 10]) -> f64 {
    // println!(
    //     r"decimal bits
    //       {:08b} {:08b} {:08b} {:08b}
    //       {:08b} {:08b} {:08b} {:08b}
    //       {:08b} {:08b}",
    //     b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9]
    // );

    let mut significand = 0f64;
    for idx in 0..8 {
        significand += read_binary_fraction(b[idx + 2], idx as u32, 0);
    }

    let is_neg = b[0] & FIRST_POS == FIRST_POS;
    let mut b = b;
    if is_neg {
        b[0] ^= FIRST_POS; // XOR the first bit, setting it to 0
    }

    match (b[0], b[1]) {
        (0, 0) => unimplemented!("cases not handled"), // ok - zero, denormal
        (0b01111111, 0b11111111) => unimplemented!("cases not handled"), // err - not supported
        // if sig > 1, 63rd bit must have been set
        (exp1, exp2) if significand.gt(&1f64) => {
            let exp = u16::from_be_bytes([exp1, exp2]);
            let res =
                2f64.powi((exp as i16 - EXP_BIAS) as i32).mul(significand);

            match is_neg {
                true => res.mul(-1f64),
                false => res,
            }
        }
        (_, _) => unimplemented!("case not handled"), // err not supported
    }
}
