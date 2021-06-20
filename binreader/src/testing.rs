use super::{BinReader, Endidness};

pub(crate) const TEST_DATA: [u8; 16] = [
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
    let mut reader = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Unknown).unwrap();
    assert_eq!(reader.current_offset(), 0);
    assert_eq!(reader.size(), TEST_DATA.len());
    assert_eq!(reader.upper_offset_limit(), TEST_DATA.len());
    assert_eq!(reader.remaining(), TEST_DATA.len());
    for i in 0..TEST_DATA.len() {
        assert_eq!(i as u8, reader.u8_at(i).unwrap());
    }
    for i in 0..TEST_DATA.len() {
        assert_eq!(reader.current_offset(), i);
        assert_eq!(reader.size(), TEST_DATA.len());
        assert_eq!(reader.remaining(), TEST_DATA.len() - i);
        assert_eq!(i as u8, reader.next_u8().unwrap());
    }
    reader = B::from_slice_with_offset(&TEST_DATA, 5, Endidness::Unknown).unwrap();
    assert_eq!(reader.current_offset(), 5);
    assert_eq!(reader.size(), TEST_DATA.len());
    assert_eq!(reader.upper_offset_limit(), TEST_DATA.len() + 5);
    assert_eq!(reader.remaining(), TEST_DATA.len());
    for i in 0..TEST_DATA.len() {
        assert_eq!(i as u8, reader.u8_at(i + 5).unwrap());
    }
    for i in 0..TEST_DATA.len() {
        assert_eq!(reader.current_offset(), i + 5);
        assert_eq!(reader.size(), TEST_DATA.len());
        assert_eq!(reader.remaining(), TEST_DATA.len() - i);
        assert_eq!(i as u8, reader.next_u8().unwrap());
    }
}

pub(crate) fn test_advance_by<'r, B: BinReader<'r>>() {
    let reader = B::from_slice(&TEST_DATA, Endidness::Unknown).unwrap();
    for i in 0..TEST_DATA.len() {
        assert_eq!(reader.current_offset(), i);
        assert_eq!(reader.size(), TEST_DATA.len());
        assert_eq!(reader.remaining(), TEST_DATA.len() - i);
        assert_eq!(reader.current_u8().unwrap(), TEST_DATA[i]);
        reader.advance_by(1).unwrap();
    }
    let reader = B::from_slice_with_offset(&TEST_DATA, 8, Endidness::Unknown).unwrap();
    for i in 0..TEST_DATA.len() {
        assert_eq!(reader.current_offset(), i + 8);
        assert_eq!(reader.size(), TEST_DATA.len());
        assert_eq!(reader.remaining(), TEST_DATA.len() - i);
        assert_eq!(reader.current_u8().unwrap(), TEST_DATA[i]);
        reader.advance_by(1).unwrap();
    }
}

pub(crate) fn test_advance_to<'r, B: BinReader<'r>>() {
    let reader = B::from_slice(&TEST_DATA, Endidness::Unknown).unwrap();
    for i in 0..TEST_DATA.len() {
        assert_eq!(reader.current_offset(), i);
        assert_eq!(reader.size(), TEST_DATA.len());
        assert_eq!(reader.remaining(), TEST_DATA.len() - i);
        assert_eq!(reader.current_u8().unwrap(), TEST_DATA[i]);
        reader.advance_to(i + 1).unwrap();
    }
    let reader = B::from_slice_with_offset(&TEST_DATA, 7, Endidness::Unknown).unwrap();
    for i in 0..TEST_DATA.len() {
        assert_eq!(reader.current_offset(), i + 7);
        assert_eq!(reader.size(), TEST_DATA.len());
        assert_eq!(reader.remaining(), TEST_DATA.len() - i);
        assert_eq!(reader.current_u8().unwrap(), TEST_DATA[i]);
        println!("\n{}", i + 1 + reader.initial_offset());
        reader.advance_to(i + 1 + reader.initial_offset()).unwrap();
        println!("{}\n", reader.current_offset());
    }
}

pub(crate) fn next_n_bytes_test<'r, B: BinReader<'r>>() {
    let reader = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Unknown).unwrap();
    let slice1 = reader.next_n_bytes(5).unwrap();
    assert_eq!(slice1, &TEST_DATA[..5]);
    assert_eq!(reader.get_remaining().unwrap(), &TEST_DATA[5..]);
    let slice2 = reader.next_n_bytes(5).unwrap();
    assert_eq!(slice2, &TEST_DATA[5..10]);
    assert_eq!(reader.get_remaining().unwrap(), &TEST_DATA[10..]);
}

pub(crate) fn basic_le_test<'r, B: BinReader<'r>>() {
    let mut reader = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Little).unwrap();
    for num in LE_U16_DATA.iter() {
        assert_eq!(*num, reader.next_u16().unwrap());
    }
    reader = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Little).unwrap();
    for num in LE_U32_DATA.iter() {
        assert_eq!(*num, reader.next_u32().unwrap());
    }
    reader = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Little).unwrap();
    for num in LE_U64_DATA.iter() {
        assert_eq!(*num, reader.next_u64().unwrap());
    }
    reader = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Little).unwrap();
    assert_eq!(LE_U128_DATA, reader.next_u128().unwrap());
}

pub(crate) fn basic_be_test<'r, B: BinReader<'r>>() {
    let mut reader = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Big).unwrap();
    for num in BE_U16_DATA.iter() {
        assert_eq!(*num, reader.next_u16().unwrap());
    }
    reader = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Big).unwrap();
    for num in BE_U32_DATA.iter() {
        assert_eq!(*num, reader.next_u32().unwrap());
    }
    reader = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Big).unwrap();
    for num in BE_U64_DATA.iter() {
        assert_eq!(*num, reader.next_u64().unwrap());
    }
    reader = B::from_slice_with_offset(&TEST_DATA, 0, Endidness::Big).unwrap();
    assert_eq!(BE_U128_DATA, reader.next_u128().unwrap());
}

pub(crate) fn test_sliced_retain_offset<'r, B: BinReader<'r>>() {
    let base_reader = B::from_slice(&TEST_DATA, Endidness::Big).unwrap();
    base_reader.advance_to(0x03).unwrap();
    let sliced_reader = base_reader.next_n_bytes_as_reader_retain_offset(5).unwrap();
    base_reader.advance_to(0x03).unwrap();
    assert_eq!(sliced_reader.initial_offset(), base_reader.current_offset());
    assert_eq!(
        sliced_reader.lower_offset_limit(),
        base_reader.current_offset()
    );
    assert_eq!(sliced_reader.current_offset(), base_reader.current_offset());
    assert_eq!(
        sliced_reader.upper_offset_limit(),
        base_reader.current_offset() + sliced_reader.size()
    );
}
