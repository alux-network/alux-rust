//! Judging a JSON value against a shape.

use alux_shape::{FieldAlg, ShapeAlg, Sorts, Spelling, Words};
use serde_json::{Map, Value};
use std::rc::Rc;

/// What a shape found where it expected something else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mismatch {
    /// Where in the value the disagreement is, as a dotted path.
    pub at: String,
    /// What the shape describes there.
    pub expected: String,
}

/// Answers whether a value is described.
pub type Verdict = Result<(), Mismatch>;

type Check = Rc<dyn Fn(&str, &Value) -> Verdict>;

/// What a leaf is, which is what decides how a writing modifier reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Datum {
    Bytes(Option<usize>),
    Int { signed: bool, bits: u16 },
    Other,
}

/// A shape, as this interpretation carries one: a check, plus what a modifier needs to know about it.
#[derive(Clone)]
pub struct Judgement {
    check: Check,
    /// The members of a product, kept so that merging one into another is expressible.
    members: Option<Vec<Member>>,
    /// Whether the value may be absent altogether, rather than merely null.
    absent_ok: bool,
    datum: Datum,
}

impl Judgement {
    /// Decides a value, reporting the first disagreement.
    ///
    /// # Errors
    ///
    /// Answers with the mismatch when the value is not the one this shape describes.
    pub fn holds(&self, value: &Value) -> Verdict {
        (self.check)("", value)
    }

    fn of(datum: Datum, check: impl Fn(&str, &Value) -> Verdict + 'static) -> Self {
        Self { check: Rc::new(check), members: None, absent_ok: false, datum }
    }

    fn leaf(check: impl Fn(&str, &Value) -> Verdict + 'static) -> Self {
        Self::of(Datum::Other, check)
    }
}

/// One member of a product, ready to be looked for in an object.
#[derive(Clone)]
pub struct Member {
    name: String,
    shape: Judgement,
}

/// Judges JSON against a shape, spelling names as the surface spells them.
#[derive(Debug, Clone, Copy)]
pub struct Judge {
    spelling: Spelling,
}

impl Judge {
    /// Judges values whose names are spelled this way.
    pub fn new(spelling: Spelling) -> Self {
        Self { spelling }
    }
}

/// Reports what was expected where.
fn wrong(at: &str, expected: impl Into<String>) -> Verdict {
    Err(Mismatch { at: if at.is_empty() { ".".into() } else { at.into() }, expected: expected.into() })
}

/// Extends a path with a member's name.
fn below(at: &str, name: &str) -> String {
    if at.is_empty() { name.into() } else { format!("{at}.{name}") }
}

/// Reads a hexadecimal string, answering with the bytes it states.
fn hex_bytes(text: &str) -> Option<usize> {
    let digits = text.strip_prefix("0x")?;

    (digits.len() % 2 == 0 && digits.chars().all(|c| c.is_ascii_hexdigit())).then_some(digits.len() / 2)
}

impl Sorts for Judge {
    type Ty = Judgement;
    type Field = Vec<Member>;
}

impl ShapeAlg for Judge {
    fn truth(&self) -> Judgement {
        Judgement::leaf(|at, v| if v.is_boolean() { Ok(()) } else { wrong(at, "a boolean") })
    }

    fn unit(&self) -> Judgement {
        Judgement::leaf(|at, v| if v.is_null() { Ok(()) } else { wrong(at, "null") })
    }

    fn text(&self) -> Judgement {
        Judgement::leaf(|at, v| if v.is_string() { Ok(()) } else { wrong(at, "text") })
    }

    fn literal(&self, text: &str) -> Judgement {
        let expected = text.to_owned();

        Judgement::leaf(move |at, v| {
            if v.as_str() == Some(expected.as_str()) { Ok(()) } else { wrong(at, format!("\"{expected}\"")) }
        })
    }

    fn name_word(&self, words: Words<'_>) -> Judgement {
        let spelled = self.spelling.spell(words);

        Judgement::leaf(
            move |at, v| {
                if v.as_str() == Some(spelled.as_str()) { Ok(()) } else { wrong(at, format!("\"{spelled}\"")) }
            },
        )
    }

    fn int(&self, signed: bool, bits: u16) -> Judgement {
        Judgement::of(Datum::Int { signed, bits }, move |at, v| {
            let fits = match v {
                Value::Number(n) if signed => n.as_i64().is_some(),
                Value::Number(n) => n.as_u64().is_some(),
                _ => false,
            };

            if fits { Ok(()) } else { wrong(at, format!("an integer of {bits} bits")) }
        })
    }

    fn float(&self, bits: u16) -> Judgement {
        Judgement::leaf(move |at, v| if v.is_number() { Ok(()) } else { wrong(at, format!("a number of {bits} bits")) })
    }

    fn bytes(&self, len: Option<usize>) -> Judgement {
        // Bytes alone say nothing about how they are written, and JSON has no form for them. A
        // writing modifier is what makes them readable, so bare bytes describe no value.
        Judgement::of(Datum::Bytes(len), |at, _| wrong(at, "bytes, with no writing stated"))
    }

    fn hex(&self, item: Judgement) -> Judgement {
        match item.datum {
            Datum::Bytes(len) => Judgement::of(item.datum, move |at, v| match v.as_str().and_then(hex_bytes) {
                Some(found) if len.is_none_or(|len| len == found) => Ok(()),
                _ => wrong(
                    at,
                    match len {
                        Some(len) => format!("hexadecimal text of {len} bytes"),
                        None => "hexadecimal text".into(),
                    },
                ),
            }),
            _ => Judgement::of(item.datum, |at, v| {
                let quantity = v.as_str().is_some_and(|text| {
                    text.strip_prefix("0x").is_some_and(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_hexdigit()))
                });

                if quantity { Ok(()) } else { wrong(at, "a hexadecimal quantity") }
            }),
        }
    }

    fn decimal(&self, item: Judgement) -> Judgement {
        Judgement::of(item.datum, |at, v| {
            let digits = v.as_str().is_some_and(|t| !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()));

            if digits { Ok(()) } else { wrong(at, "decimal digits in text") }
        })
    }

    fn base64(&self, item: Judgement) -> Judgement {
        Judgement::of(item.datum, |at, v| if v.is_string() { Ok(()) } else { wrong(at, "base64 text") })
    }

    fn opt(&self, item: Judgement) -> Judgement {
        let inner = item.check.clone();
        let mut shape = Judgement::of(item.datum, move |at, v| if v.is_null() { Ok(()) } else { inner(at, v) });
        shape.absent_ok = true;

        shape
    }

    fn seq(&self, item: Judgement) -> Judgement {
        let inner = item.check.clone();

        Judgement::leaf(move |at, v| match v.as_array() {
            Some(items) => items.iter().enumerate().try_for_each(|(i, item)| inner(&below(at, &i.to_string()), item)),
            None => wrong(at, "a sequence"),
        })
    }

    fn map(&self, _key: Judgement, value: Judgement) -> Judgement {
        let inner = value.check.clone();

        Judgement::leaf(move |at, v| match v.as_object() {
            Some(entries) => entries.iter().try_for_each(|(k, v)| inner(&below(at, k), v)),
            None => wrong(at, "an association"),
        })
    }

    fn product(&self, fields: Vec<Vec<Member>>) -> Judgement {
        let members: Vec<Member> = fields.into_iter().flatten().collect();
        let described = members.clone();
        let mut shape = Judgement::leaf(move |at, v| judge_product(at, v, &described));
        shape.members = Some(members);

        shape
    }

    fn choice(&self, alternatives: Vec<Judgement>) -> Judgement {
        let checks: Vec<Check> = alternatives.iter().map(|a| a.check.clone()).collect();

        Judgement::leaf(move |at, v| {
            if checks.iter().any(|check| check(at, v).is_ok()) {
                Ok(())
            } else {
                wrong(at, format!("one of {} alternatives", checks.len()))
            }
        })
    }

    fn named(&self, _words: Words, body: Judgement) -> Judgement {
        // A name is an identity, and identity does not decide a value.
        body
    }

    fn reference(&self, words: Words<'_>) -> Judgement {
        // Resolving a name needs the whole term, which a fold does not hold. Until a term is read
        // back from its written form, a reference describes anything.
        let _ = words;

        Judgement::leaf(|_, _| Ok(()))
    }
}

/// Decides an object against the members described for it, in both directions.
fn judge_product(at: &str, value: &Value, members: &[Member]) -> Verdict {
    let Some(entries) = value.as_object() else {
        return wrong(at, "an object");
    };

    for member in members {
        match entries.get(&member.name) {
            Some(found) => (member.shape.check)(&below(at, &member.name), found)?,
            None if member.shape.absent_ok => (),
            None => return wrong(&below(at, &member.name), "a member that is present"),
        }
    }

    undescribed(entries, members)
        .map_or(Ok(()), |name| wrong(&below(at, &name), "no member, since the shape describes none here"))
}

/// Names a key the shape does not describe, if the value carries one.
fn undescribed(entries: &Map<String, Value>, members: &[Member]) -> Option<String> {
    entries.keys().find(|key| !members.iter().any(|m| &&m.name == key)).cloned()
}

impl FieldAlg for Judge {
    fn field(&self, words: Words, shape: Judgement) -> Vec<Member> {
        vec![Member { name: self.spelling.spell(words), shape }]
    }

    fn merge(&self, shape: Judgement) -> Vec<Member> {
        // Merging answers with the members of the product merged in, so a merge of anything else
        // contributes a member no value can satisfy.
        shape.members.clone().unwrap_or_else(|| {
            vec![Member { name: String::new(), shape: Judgement::leaf(|at, _| wrong(at, "a product, to merge")) }]
        })
    }
}
