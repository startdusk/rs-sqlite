use binary_serde::{BinarySerde, Endianness};

#[derive(BinarySerde, Debug)]
#[repr(C)]
struct Row {
    id: u32,
    username: [u8; 32],
    email: [u8; 255],
}

fn main() {
    let row = Row {
        id: 12,
        username: [1u8; 32],
        email: [1u8; 255],
    };

    let mut buffer = [0u8; <Row as BinarySerde>::MAX_SERIALIZED_SIZE];
    row.binary_serialize(&mut buffer, Endianness::Big);
    println!("{:?}", buffer);
    let decode = Row::binary_deserialize(&buffer, Endianness::Big).unwrap();
    println!("{:?}", decode)
}
