extern crate code128_encoder;

fn assert_encodes_to(s: &str, payload: &str) {
    assert_eq!(code128_encoder::encode(String::from(s)), payload);
}

#[test]
fn test_basic_digit_string() {
    assert_encodes_to("0123456789", ">;0123456789");
}

#[test]
fn test_four_leading_digits() {
    assert_encodes_to("1234ABC", ">;1234>6ABC");
}

#[test]
fn test_fewer_than_four_leading_digits() {
    assert_encodes_to("123ABC", ">:123ABC");
}

#[test]
fn test_odd_leading_digits() {
    assert_encodes_to("12345ABC", ">;1234>65ABC");
}

#[test]
fn test_six_consecutive_digits() {
    assert_encodes_to("ABC123456ABC", ">:ABC>5123456>6ABC");
}

#[test]
fn test_fewer_than_six_consecutive_digits() {
    assert_encodes_to("ABC12345ABC", ">:ABC12345ABC");
}

#[test]
fn test_odd_consecutive_digits() {
    assert_encodes_to("ABC1234567ABC", ">:ABC1>5234567>6ABC");
}

#[test]
fn test_four_trailing_digits() {
    assert_encodes_to("ABC1234", ">:ABC>51234");
}

#[test]
fn test_odd_digits() {
    assert_encodes_to("1234567", ">;123456>67");
}
