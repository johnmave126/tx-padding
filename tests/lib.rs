//! Test the padding scheme of different block sizes and message length
#![no_std]

use typenum::marker_traits::Unsigned;

use tx_padding::consts;
use tx_padding::{Padding, TxPadding};

macro_rules! create_passing_test {
    ($name:ident, $block_size:ty, $buf_len:expr, $message:expr, $padding_start:expr, $padded_msg:expr) => {
        #[test]
        fn $name() {
            let message = $message;
            let mut buf = [0; $buf_len];
            let n = message.len();
            buf[..n].copy_from_slice(message);
            let padded_msg =
                TxPadding::<$block_size>::pad(&mut buf, n, <$block_size>::to_usize()).unwrap();
            assert_eq!(&padded_msg[$padding_start..], $padded_msg);
            assert_eq!(
                (padded_msg[0] & (<$block_size>::to_u8() - 1)) as usize + 2,
                $padding_start - 1
            );
            assert_eq!(
                TxPadding::<$block_size>::unpad(&padded_msg).unwrap(),
                message
            );
        }
    };
}

create_passing_test!(padding_bs_2_msg_0, consts::U2, 4, b"", 3, b"\x00");
create_passing_test!(padding_bs_2_msg_1, consts::U2, 6, b"\x01", 4, b"\x01\x00");
create_passing_test!(
    padding_bs_2_msg_2,
    consts::U2,
    6,
    b"\x01\x02",
    3,
    b"\x01\x02\x00"
);
create_passing_test!(
    padding_bs_2_msg_3,
    consts::U2,
    8,
    b"\x01\x02\x03",
    4,
    b"\x01\x02\x03\x00"
);

create_passing_test!(padding_bs_4_msg_0, consts::U4, 8, b"", 5, b"\x00\x00\x00");
create_passing_test!(
    padding_bs_4_msg_1,
    consts::U4,
    8,
    b"\x01",
    4,
    b"\x01\x00\x00\x00"
);
create_passing_test!(
    padding_bs_4_msg_2,
    consts::U4,
    8,
    b"\x01\x02",
    3,
    b"\x01\x02\x00\x00\x00"
);
create_passing_test!(
    padding_bs_4_msg_3,
    consts::U4,
    12,
    b"\x01\x02\x03",
    6,
    b"\x01\x02\x03\x00\x00\x00"
);
create_passing_test!(
    padding_bs_4_msg_4,
    consts::U4,
    12,
    b"\x01\x02\x03\x04",
    5,
    b"\x01\x02\x03\x04\x00\x00\x00"
);

create_passing_test!(
    padding_bs_8_msg_0,
    consts::U8,
    16,
    b"",
    9,
    b"\x00\x00\x00\x00\x00\x00\x00"
);
create_passing_test!(
    padding_bs_8_msg_1,
    consts::U8,
    16,
    b"\x01",
    8,
    b"\x01\x00\x00\x00\x00\x00\x00\x00"
);
create_passing_test!(
    padding_bs_8_msg_5,
    consts::U8,
    16,
    b"\x01\x02\x03\x04\x05",
    4,
    b"\x01\x02\x03\x04\x05\x00\x00\x00\x00\x00\x00\x00"
);
create_passing_test!(
    padding_bs_8_msg_7,
    consts::U8,
    24,
    b"\x01\x02\x03\x04\x05\x06\x07",
    10,
    b"\x01\x02\x03\x04\x05\x06\x07\x00\x00\x00\x00\x00\x00\x00"
);
create_passing_test!(
    padding_bs_8_msg_7_longer_buf,
    consts::U8,
    25,
    b"\x01\x02\x03\x04\x05\x06\x07",
    10,
    b"\x01\x02\x03\x04\x05\x06\x07\x00\x00\x00\x00\x00\x00\x00"
);

#[test]
fn reject_insufficient_space() {
    let message = b"\x01\x02\x03";
    let mut buf = [0; 15];
    let n = message.len();
    buf[..n].copy_from_slice(message);
    assert!(TxPadding::<consts::U8>::pad(&mut buf, n, 8).is_err());
}

#[test]
fn reject_mismatch_size() {
    let message = b"\x01\x02\x03";
    let mut buf = [0; 16];
    let n = message.len();
    buf[..n].copy_from_slice(message);
    assert!(TxPadding::<consts::U8>::pad(&mut buf, n, 4).is_err());
    assert!(TxPadding::<consts::U8>::pad(&mut buf, n, 8).is_ok());
}

#[test]
fn reject_illformed_padded_message() {
    assert!(TxPadding::<consts::U8>::unpad(&[]).is_err());
    assert!(TxPadding::<consts::U8>::unpad(&[0xF8]).is_err());
    assert!(TxPadding::<consts::U8>::unpad(&[0xF8, 0, 0]).is_err());
    assert!(
        TxPadding::<consts::U8>::unpad(&[0xF8, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0])
            .is_err()
    );
}
