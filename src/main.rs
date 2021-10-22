use bincode::encode_into_slice;

#[derive(Debug, Default, bincode::Encode, bincode::Decode, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Default, bincode::Encode, bincode::Decode, PartialEq)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Default, bincode::Encode, bincode::Decode, PartialEq)]
pub struct Rect {
    pub point: Point,
    pub size: Size,
}

#[derive(Debug, Default, bincode::Encode, bincode::Decode, PartialEq)]
pub struct CommonItemProperties {
    pub clip_rect: Rect,
}

fn main() {
    let mut buffer = vec![0u8;1024];
    let config = bincode::config::Configuration::standard();
    for i in 1..10000000 {
        let my_struct = CommonItemProperties {
            clip_rect: Rect {
                point: Point { x: 1.0, y: 2.0 },
                size: Size { w: 4.0, h: 5.0 },
            },
        };
        bincode::encode_into_slice(&my_struct, &mut buffer.as_mut_slice(), config).unwrap();
    }
}

