// Copyright (c) 2017-2023, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use rust_decimal::Decimal;

pub fn round_d128_generalized(to_round: &Decimal, places_past_decimal: u32) -> Decimal {
    let rounded: Decimal = to_round.round_dp(places_past_decimal);
    rounded//.reduce()
}

pub fn round_d128_1e2(to_round: &Decimal) -> Decimal {
    let rounded: Decimal = to_round.round_dp(2);
    rounded//.reduce()
}

pub fn round_d128_1e8(to_round: &Decimal) -> Decimal {
    let rounded: Decimal = to_round.round_dp(8);
    rounded//.reduce()
        //  Note: quantize() rounds the number to the right of decimal and keeps it, discarding the rest to the right (it appears). See test.
        //  In other words, it's off by one. If you raise 0.123456789 by 10e8, quantize to 1e1 (which is 10), it'll get 12345678.9, round off to 12345679, and end up .12345679
        //  If you quantize the same number to 1e2 (which is 100), it starts back toward the left, so it'll get 12345678.9, round off to 12345680
        //  If you quantize the same number to 1e3 (which is 1000), it starts back toward the left, so it'll get 12345678.9, round off to 12345700
        //  As you can see, the quantize is off by one.  Quantizing to 10 rounds off the nearest one.  Quantizing to 100 rounds off to nearest 10, etc.
}