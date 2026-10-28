use alux_shape::{Product, ProductLawsExt, Shape, Sum, Sum3, SumLawsExt};

#[derive(Shape, Clone, Debug, PartialEq)]
struct Credentials<User> {
    user: User,
    #[serde(rename = "passPhrase")]
    secret: String,
    #[serde(skip)]
    attempts: u8,
}

#[derive(Shape, Clone, Debug, PartialEq)]
enum Target<Location> {
    At(Location),
    #[serde(rename = "url")]
    Remote(String),
    StandardOut,
}

#[test]
fn a_product_names_its_fields_by_the_words_its_shape_reads() {
    assert_eq!(Credentials::<u8>::FIELDS, ["user", "pass_phrase"]);
    let credentials = Credentials::from_fields((7, "open".to_owned()));
    assert_eq!(credentials, Credentials { user: 7, secret: "open".to_owned(), attempts: 0 });
    assert_eq!(credentials.clone().into_fields(), (7, "open".to_owned()));
    assert!(credentials.law_product_round_trip());
}

#[test]
fn a_sum_names_its_alternatives_and_carries_exactly_one() {
    assert_eq!(Target::<u8>::ALTERNATIVES, ["at", "url", "standard_out"]);
    assert_eq!(Target::<u8>::from_alternative(Sum3::C(())), Target::StandardOut);
    assert_eq!(Target::At(3).into_alternative(), Sum3::A(3));
    for target in [Target::At(3), Target::Remote("x".to_owned()), Target::StandardOut] {
        assert!(target.law_sum_round_trip());
    }
}

#[derive(Shape, Clone, Debug, PartialEq)]
enum Hex {
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
    Da,
    Db,
    Dc,
    Dd,
    De,
    Df(u8),
}

#[test]
fn a_sum_holds_up_to_sixteen_alternatives() {
    assert_eq!(Hex::ALTERNATIVES.len(), 16);
    assert_eq!(Hex::Df(1).into_alternative(), alux_shape::Sum16::P(1));
    assert!(Hex::De.law_sum_round_trip());
}
