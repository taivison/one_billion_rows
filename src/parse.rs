use std::simd::{i16x4, num::SimdInt};

const ZEROES: i16x4 = i16x4::splat(b'0' as i16);
const MUL_MASK: i16x4 = i16x4::from_array([100, 10, 0, 1]);

pub fn parse_temp(temp: &[u8]) -> i16 {
    if temp.is_empty() {
        return 0;
    }

    let sign = if temp[0] == b'-' { -1i16 } else { 1i16 };
    let start = (temp[0] == b'-') as usize;

    let digits = &temp[start..];
    let mut buf = [b'0'; 4];

    buf[4 - digits.len()..].copy_from_slice(digits);

    let mut s = i16x4::from_array(buf.map(|c| c as i16));

    s -= ZEROES;
    s *= MUL_MASK;

    s.reduce_sum() * sign
}

#[cfg(test)]
mod test {
    use crate::parse::parse_temp;

    macro_rules! temp_tests {
    ( $( $name:ident : $input:expr => $expected:expr ),+ $(,)? ) => {
        $(
            #[test]
            fn $name() {
                let input: &[u8] = $input;
                let got = parse_temp(input);
                assert_eq!(got, $expected, "input = {:?}", input);
            }
        )+
    };
}

    temp_tests! {
        test_1: b"99.9" => 999,
        test_2: b"-99.9" => -999,
        test_3: b"12.3" => 123,
        test_4: b"-7.2" => -72,
    }
}
