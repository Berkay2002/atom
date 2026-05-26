//! Six colormaps as 8-stop reference points. Linearly interpolated into a
//! 256-entry RGBA8 1D texture at upload time. Stops sampled by eye from
//! matplotlib's perceptually-uniform maps; tables are 0..=255 RGB.

pub const INFERNO: [[u8; 3]; 8] = [
    [0, 0, 4], [40, 11, 84], [101, 21, 110], [159, 42, 99],
    [212, 72, 66], [245, 125, 21], [250, 193, 39], [252, 255, 164],
];

pub const VIRIDIS: [[u8; 3]; 8] = [
    [68, 1, 84], [72, 40, 120], [62, 73, 137], [49, 104, 142],
    [38, 130, 142], [31, 158, 137], [53, 183, 121], [253, 231, 37],
];

pub const MAGMA: [[u8; 3]; 8] = [
    [0, 0, 4], [27, 12, 65], [80, 18, 123], [135, 35, 138],
    [192, 56, 130], [240, 96, 93], [253, 159, 109], [252, 253, 191],
];

pub const PLASMA: [[u8; 3]; 8] = [
    [13, 8, 135], [75, 3, 161], [125, 3, 168], [168, 34, 150],
    [203, 70, 121], [229, 107, 93], [248, 148, 65], [240, 249, 33],
];

pub const GRAYSCALE: [[u8; 3]; 8] = [
    [0, 0, 0], [36, 36, 36], [72, 72, 72], [109, 109, 109],
    [145, 145, 145], [182, 182, 182], [218, 218, 218], [255, 255, 255],
];

pub const ELECTRON_BLUE: [[u8; 3]; 8] = [
    [0, 0, 0], [4, 16, 48], [8, 40, 96], [16, 80, 144],
    [40, 144, 192], [120, 200, 224], [200, 240, 248], [255, 255, 255],
];

pub const ALL: &[(&str, &[[u8; 3]])] = &[
    ("inferno", &INFERNO),
    ("viridis", &VIRIDIS),
    ("magma", &MAGMA),
    ("plasma", &PLASMA),
    ("grayscale", &GRAYSCALE),
    ("electron-blue", &ELECTRON_BLUE),
];
