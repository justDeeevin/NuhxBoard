pub fn code_convert(xinput_code: u32) -> u32 {
    // mouse buttons
    match xinput_code {
        // left
        1 => return 0,
        // right
        3 => return 1,
        //Scroll Down
        4 => return 22,
        //Scroll up
        5 => return 21,
        _ => {}
    };
    // 1-9
    if (10..=18).contains(&xinput_code) {
        return xinput_code + 39;
    }
    // bckspce + tab
    if (22..=23).contains(&xinput_code) {
        return xinput_code - 14;
    }

    match xinput_code {
        // 0
        19 => 48,
        // -
        20 => 189,
        // =
        21 => 187,
        // q
        24 => 81,
        // w
        25 => 87,
        // e
        26 => 69,
        // r
        27 => 82,
        // t
        28 => 84,
        // y
        29 => 89,
        // u
        30 => 85,
        // i
        31 => 73,
        // o
        32 => 79,
        // p
        33 => 80,
        // [
        34 => 219,
        // ]
        35 => 221,
        // enter
        36 => 13,
        // ctrl
        37 => 162,
        // a
        38 => 65,
        // s
        39 => 83,
        // d
        40 => 68,
        // f
        41 => 70,
        // g
        42 => 71,
        // h
        43 => 72,
        // j
        44 => 74,
        // k
        45 => 75,
        // l
        46 => 76,
        // ;
        47 => 186,
        // '
        48 => 222,
        // `
        49 => 192,
        // shift
        50 => 160,
        // \
        51 => 220,
        // z
        52 => 90,
        // x
        53 => 88,
        // c
        54 => 67,
        // v
        55 => 86,
        // b
        56 => 66,
        // n
        57 => 78,
        // m
        58 => 77,
        // ,
        59 => 188,
        // .
        60 => 190,
        // /
        61 => 191,
        // shift
        62 => 161,
        // *
        63 => 106,
        // alt
        64 => 18,
        // space
        65 => 32,
        // caps
        66 => 20,
        _ => panic!("Unknown xinput code: {}", xinput_code),
    }
}
