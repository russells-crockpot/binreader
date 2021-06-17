use super::{BinReader, Endidness};

pub const TEST_DATA: [u8; 16] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
];

const LE_U16_DATA: [u16; 8] = [
    0x0100, 0x0302, 0x0504, 0x0706, 0x0908, 0x0b0a, 0x0d0c, 0x0f0e,
];
const BE_U16_DATA: [u16; 8] = [
    0x0001, 0x0203, 0x0405, 0x0607, 0x0809, 0x0a0b, 0x0c0d, 0x0e0f,
];

const LE_U32_DATA: [u32; 4] = [0x03020100, 0x07060504, 0x0b0a0908, 0x0f0e0d0c];
const BE_U32_DATA: [u32; 4] = [0x00010203, 0x04050607, 0x08090a0b, 0x0c0d0e0f];

const LE_U64_DATA: [u64; 2] = [0x0706050403020100, 0x0f0e0d0c0b0a0908];
const BE_U64_DATA: [u64; 2] = [0x0001020304050607, 0x08090a0b0c0d0e0f];

const LE_U128_DATA: u128 = 0x0f0e0d0c0b0a09080706050403020100;
const BE_U128_DATA: u128 = 0x000102030405060708090a0b0c0d0e0f;

pub(crate) fn basic_test_1<'r, B: BinReader<'r>>() {
    let mut data = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Unknown).unwrap();
    assert_eq!(data.current_offset(), 0);
    assert_eq!(data.size(), TEST_DATA.len());
    assert_eq!(data.upper_offset_limit(), TEST_DATA.len());
    assert_eq!(data.remaining(), TEST_DATA.len());
    for i in 0..TEST_DATA.len() {
        assert_eq!(i as u8, data.u8_at(i).unwrap());
    }
    for i in 0..TEST_DATA.len() {
        assert_eq!(data.current_offset(), i);
        assert_eq!(data.size(), TEST_DATA.len());
        assert_eq!(data.remaining(), TEST_DATA.len() - i);
        assert_eq!(i as u8, data.next_u8().unwrap());
    }
    data = B::from_slice_with_offset(&TEST_DATA, 5, Endidness::Unknown).unwrap();
    assert_eq!(data.current_offset(), 5);
    assert_eq!(data.size(), TEST_DATA.len());
    assert_eq!(data.upper_offset_limit(), TEST_DATA.len() + 5);
    assert_eq!(data.remaining(), TEST_DATA.len());
    for i in 0..TEST_DATA.len() {
        assert_eq!(i as u8, data.u8_at(i + 5).unwrap());
    }
    for i in 0..TEST_DATA.len() {
        assert_eq!(data.current_offset(), i + 5);
        assert_eq!(data.size(), TEST_DATA.len());
        assert_eq!(data.remaining(), TEST_DATA.len() - i);
        assert_eq!(i as u8, data.next_u8().unwrap());
    }
}

pub(crate) fn basic_le_test<'r, B: BinReader<'r>>() {
    let mut data = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Little).unwrap();
    for num in LE_U16_DATA.iter() {
        assert_eq!(*num, data.next_u16().unwrap());
    }
    data = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Little).unwrap();
    for num in LE_U32_DATA.iter() {
        assert_eq!(*num, data.next_u32().unwrap());
    }
    data = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Little).unwrap();
    for num in LE_U64_DATA.iter() {
        assert_eq!(*num, data.next_u64().unwrap());
    }
    data = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Little).unwrap();
    assert_eq!(LE_U128_DATA, data.next_u128().unwrap());
}

pub(crate) fn basic_be_test<'r, B: BinReader<'r>>() {
    let mut data = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Big).unwrap();
    for num in BE_U16_DATA.iter() {
        assert_eq!(*num, data.next_u16().unwrap());
    }
    data = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Big).unwrap();
    for num in BE_U32_DATA.iter() {
        assert_eq!(*num, data.next_u32().unwrap());
    }
    data = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Big).unwrap();
    for num in BE_U64_DATA.iter() {
        assert_eq!(*num, data.next_u64().unwrap());
    }
    data = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Big).unwrap();
    assert_eq!(BE_U128_DATA, data.next_u128().unwrap());
}
