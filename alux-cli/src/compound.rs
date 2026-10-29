use crate::{
    ArgumentInfo, CliArgumentError, CliDecodeSources, CliNamedSource, CliNamedSources, CliSourceDecode, CliSourceFor,
    CliSources, CliWordsAlg, Optional, Part, Whole, nested, nested_pattern, source::words,
};
use alux_shape::{
    Product, Sum, Sum2, Sum3, Sum4, Sum5, Sum6, Sum7, Sum8, Sum9, Sum10, Sum11, Sum12, Sum13, Sum14, Sum15, Sum16,
};
use std::marker::PhantomData;

/// Declares the fields of a product, one named source each, in the order of the product's fields.
#[derive(Clone, Copy, Debug)]
pub struct ProductSource<Fields, Presence = Whole> {
    fields: Fields,
    presence: PhantomData<Presence>,
}

/// Declares the alternatives of a sum, one named source each, in the order of the sum's.
#[derive(Clone, Copy, Debug)]
pub struct SumSource<Alternatives, Presence = Whole> {
    alternatives: Alternatives,
    presence: PhantomData<Presence>,
}

impl ProductSource<()> {
    /// Starts a product with no fields.
    pub fn new() -> Self {
        Self { fields: (), presence: PhantomData }
    }
}

impl Default for ProductSource<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Fields, Presence> ProductSource<Fields, Presence> {
    /// Declares the source of the next field.
    pub fn field<Source>(self, source: Source) -> ProductSource<(Fields, Source), Presence>
    where
        Source: CliNamedSource,
    {
        ProductSource { fields: (self.fields, source), presence: PhantomData }
    }

    /// Supplies `Option` of the product: all of its fields or none of them.
    pub fn optional(self) -> ProductSource<Fields, Optional> {
        ProductSource { fields: self.fields, presence: PhantomData }
    }
}

impl SumSource<()> {
    /// Starts a sum with no alternatives.
    pub fn new() -> Self {
        Self { alternatives: (), presence: PhantomData }
    }
}

impl Default for SumSource<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Alternatives, Presence> SumSource<Alternatives, Presence> {
    /// Declares the source of the next alternative.
    pub fn or<Source>(self, source: Source) -> SumSource<(Alternatives, Source), Presence>
    where
        Source: CliNamedSource,
    {
        SumSource { alternatives: (self.alternatives, source), presence: PhantomData }
    }

    /// Supplies `Option` of the sum: at most one alternative.
    pub fn optional(self) -> SumSource<Alternatives, Optional> {
        SumSource { alternatives: self.alternatives, presence: PhantomData }
    }
}

/// Marks described arguments as the parts of the compound argument `name`.
fn parts(name: &'static str, part: fn(&'static str) -> Part, mut arguments: Vec<ArgumentInfo>) -> Vec<ArgumentInfo> {
    for argument in &mut arguments {
        argument.part_of = Some(part(name));
    }
    arguments
}

impl<Value, Fields> CliSourceFor<Value> for ProductSource<Fields, Whole>
where
    Value: Product,
    Fields: CliSources<Value::Fields> + CliNamedSources,
{
    fn cli_describe(&self, name: &'static str, _: &mut usize, out: &mut Vec<ArgumentInfo>) {
        out.extend(parts(name, Part::Product, self.fields.cli_arguments(Value::FIELDS)));
    }
}

impl<Value, Fields> CliSourceFor<Option<Value>> for ProductSource<Fields, Optional>
where
    Value: Product,
    Fields: CliSources<Value::Fields> + CliNamedSources,
{
    fn cli_describe(&self, name: &'static str, _: &mut usize, out: &mut Vec<ArgumentInfo>) {
        out.extend(parts(name, Part::Product, self.fields.cli_arguments(Value::FIELDS)));
    }
}

impl<Value, Fields, Decoder, Error> CliSourceDecode<Value, Decoder, Error> for ProductSource<Fields, Whole>
where
    Value: Product,
    Fields: CliDecodeSources<Value::Fields, Decoder, Error>,
{
    fn cli_decode_source(
        &self,
        arguments: &mut &[ArgumentInfo],
        decoder: &Decoder,
    ) -> Result<Value, CliArgumentError<Error>> {
        self.fields.cli_decode(take(arguments, Value::FIELDS.len()), decoder).map(Value::from_fields)
    }
}

/// An optional product takes all of its fields when the command line gives any of them, and is
/// `None` when it gives none and no fallback answers.
impl<Value, Fields, Decoder, Error> CliSourceDecode<Option<Value>, Decoder, Error> for ProductSource<Fields, Optional>
where
    Value: Product,
    Fields: CliSources<Value::Fields> + CliDecodeSources<Value::Fields, Decoder, Error>,
    Decoder: CliWordsAlg,
{
    fn cli_decode_source(
        &self,
        arguments: &mut &[ArgumentInfo],
        decoder: &Decoder,
    ) -> Result<Option<Value>, CliArgumentError<Error>> {
        let fields = take(arguments, Value::FIELDS.len());
        if fields.iter().all(|field| decoder.cli_words(field).is_empty()) {
            return Ok(None);
        }
        self.fields.cli_decode(fields, decoder).map(|fields| Some(Value::from_fields(fields)))
    }
}

/// Takes the next `count` described arguments: a compound source's parts, one each.
fn take<'a>(arguments: &mut &'a [ArgumentInfo], count: usize) -> &'a [ArgumentInfo] {
    let (parts, rest) = arguments.split_at(count);
    *arguments = rest;
    parts
}

/// Answers the one alternative given, by index, or why not exactly one was.
fn chosen<Error>(arguments: &[ArgumentInfo], given: &[bool]) -> Result<Option<usize>, CliArgumentError<Error>> {
    let mut chosen = given.iter().enumerate().filter(|(_, given)| **given).map(|(index, _)| index);
    match (chosen.next(), chosen.next()) {
        (None, _) => Ok(None),
        (Some(index), None) => Ok(Some(index)),
        (Some(first), Some(second)) => Err(CliArgumentError::Conflict(arguments[first].name, arguments[second].name)),
    }
}

macro_rules! sum {
    ($sum:ident; $(($source:ident, $alternative:ident, $value:ident, $variant:ident, $index:tt)),+) => {
        impl<Presence, $($source),+> SumSource<nested!((); $($source)+), Presence> {
            /// Describes the alternatives of the sum `Value`, as parts of the argument `name`.
            fn describe<Value, $($alternative),+>(&self, name: &'static str) -> Vec<ArgumentInfo>
            where
                Value: Sum<Alternatives = $sum<$($alternative),+>>,
                nested!((); $($source)+): CliSources<($($alternative,)+)>,
            {
                parts(name, Part::Sum, self.alternatives.cli_arguments(Value::ALTERNATIVES))
            }

            /// Answers the one alternative given, if any, or why more than one was.
            fn chosen_value<Value, Decoder, Error, $($alternative),+>(
                &self,
                arguments: &mut &[ArgumentInfo],
                decoder: &Decoder,
            ) -> Result<Option<Value>, CliArgumentError<Error>>
            where
                Value: Sum<Alternatives = $sum<$($alternative),+>>,
                Decoder: CliWordsAlg,
                $($source: CliSourceDecode<$alternative, Decoder, Error>,)+
            {
                let arguments = take(arguments, Value::ALTERNATIVES.len());
                let given = arguments.iter().map(|argument| !words(argument, decoder).is_empty()).collect::<Vec<_>>();
                let nested_pattern!((); $($value)+) = &self.alternatives;
                Ok(match chosen(arguments, &given)? {
                    None => None,
                    $(Some($index) => {
                        let value = $value.cli_decode_source(&mut &arguments[$index..], decoder)?;
                        Some(Value::from_alternative($sum::$variant(value)))
                    })+
                    Some(_) => unreachable!("a sum chose an alternative it does not have"),
                })
            }
        }

        impl<Value, $($source, $alternative),+> CliSourceFor<Value> for SumSource<nested!((); $($source)+), Whole>
        where
            Value: Sum<Alternatives = $sum<$($alternative),+>>,
            nested!((); $($source)+): CliSources<($($alternative,)+)> + CliNamedSources,
        {
            fn cli_describe(&self, name: &'static str, _: &mut usize, out: &mut Vec<ArgumentInfo>) {
                out.extend(self.describe::<Value, $($alternative),+>(name));
            }
        }

        impl<Value, $($source, $alternative),+> CliSourceFor<Option<Value>> for SumSource<nested!((); $($source)+), Optional>
        where
            Value: Sum<Alternatives = $sum<$($alternative),+>>,
            nested!((); $($source)+): CliSources<($($alternative,)+)> + CliNamedSources,
        {
            fn cli_describe(&self, name: &'static str, _: &mut usize, out: &mut Vec<ArgumentInfo>) {
                out.extend(self.describe::<Value, $($alternative),+>(name));
            }
        }

        impl<Value, Decoder, Error, $($source, $alternative),+> CliSourceDecode<Value, Decoder, Error>
            for SumSource<nested!((); $($source)+), Whole>
        where
            Value: Sum<Alternatives = $sum<$($alternative),+>>,
            nested!((); $($source)+): CliSources<($($alternative,)+)>,
            Decoder: CliWordsAlg,
            $($source: CliSourceDecode<$alternative, Decoder, Error>,)+
        {
            fn cli_decode_source(
                &self,
                arguments: &mut &[ArgumentInfo],
                decoder: &Decoder,
            ) -> Result<Value, CliArgumentError<Error>> {
                // A sum given no alternative is missing as the argument its alternatives are parts of.
                let name = match arguments.first().and_then(|argument| argument.part_of) {
                    Some(Part::Sum(name) | Part::Product(name)) => name,
                    None => "",
                };
                self.chosen_value::<Value, Decoder, Error, $($alternative),+>(arguments, decoder)?.ok_or(CliArgumentError::Missing(name))
            }
        }

        impl<Value, Decoder, Error, $($source, $alternative),+> CliSourceDecode<Option<Value>, Decoder, Error>
            for SumSource<nested!((); $($source)+), Optional>
        where
            Value: Sum<Alternatives = $sum<$($alternative),+>>,
            nested!((); $($source)+): CliSources<($($alternative,)+)>,
            Decoder: CliWordsAlg,
            $($source: CliSourceDecode<$alternative, Decoder, Error>,)+
        {
            fn cli_decode_source(
                &self,
                arguments: &mut &[ArgumentInfo],
                decoder: &Decoder,
            ) -> Result<Option<Value>, CliArgumentError<Error>> {
                self.chosen_value::<Value, Decoder, Error, $($alternative),+>(arguments, decoder)
            }
        }
    };
}

sum!(Sum2; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1));
sum!(Sum3; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1), (S2, A2, s2, C, 2));
sum!(Sum4; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1), (S2, A2, s2, C, 2), (S3, A3, s3, D, 3));
sum!(Sum5; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1), (S2, A2, s2, C, 2), (S3, A3, s3, D, 3), (S4, A4, s4, E, 4));
sum!(Sum6; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1), (S2, A2, s2, C, 2), (S3, A3, s3, D, 3), (S4, A4, s4, E, 4), (S5, A5, s5, F, 5));
sum!(Sum7; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1), (S2, A2, s2, C, 2), (S3, A3, s3, D, 3), (S4, A4, s4, E, 4), (S5, A5, s5, F, 5), (S6, A6, s6, G, 6));
sum!(Sum8; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1), (S2, A2, s2, C, 2), (S3, A3, s3, D, 3), (S4, A4, s4, E, 4), (S5, A5, s5, F, 5), (S6, A6, s6, G, 6), (S7, A7, s7, H, 7));
sum!(Sum9; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1), (S2, A2, s2, C, 2), (S3, A3, s3, D, 3), (S4, A4, s4, E, 4), (S5, A5, s5, F, 5), (S6, A6, s6, G, 6), (S7, A7, s7, H, 7), (S8, A8, s8, I, 8));
sum!(Sum10; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1), (S2, A2, s2, C, 2), (S3, A3, s3, D, 3), (S4, A4, s4, E, 4), (S5, A5, s5, F, 5), (S6, A6, s6, G, 6), (S7, A7, s7, H, 7), (S8, A8, s8, I, 8), (S9, A9, s9, J, 9));
sum!(Sum11; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1), (S2, A2, s2, C, 2), (S3, A3, s3, D, 3), (S4, A4, s4, E, 4), (S5, A5, s5, F, 5), (S6, A6, s6, G, 6), (S7, A7, s7, H, 7), (S8, A8, s8, I, 8), (S9, A9, s9, J, 9), (S10, A10, s10, K, 10));
sum!(Sum12; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1), (S2, A2, s2, C, 2), (S3, A3, s3, D, 3), (S4, A4, s4, E, 4), (S5, A5, s5, F, 5), (S6, A6, s6, G, 6), (S7, A7, s7, H, 7), (S8, A8, s8, I, 8), (S9, A9, s9, J, 9), (S10, A10, s10, K, 10), (S11, A11, s11, L, 11));
sum!(Sum13; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1), (S2, A2, s2, C, 2), (S3, A3, s3, D, 3), (S4, A4, s4, E, 4), (S5, A5, s5, F, 5), (S6, A6, s6, G, 6), (S7, A7, s7, H, 7), (S8, A8, s8, I, 8), (S9, A9, s9, J, 9), (S10, A10, s10, K, 10), (S11, A11, s11, L, 11), (S12, A12, s12, M, 12));
sum!(Sum14; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1), (S2, A2, s2, C, 2), (S3, A3, s3, D, 3), (S4, A4, s4, E, 4), (S5, A5, s5, F, 5), (S6, A6, s6, G, 6), (S7, A7, s7, H, 7), (S8, A8, s8, I, 8), (S9, A9, s9, J, 9), (S10, A10, s10, K, 10), (S11, A11, s11, L, 11), (S12, A12, s12, M, 12), (S13, A13, s13, N, 13));
sum!(Sum15; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1), (S2, A2, s2, C, 2), (S3, A3, s3, D, 3), (S4, A4, s4, E, 4), (S5, A5, s5, F, 5), (S6, A6, s6, G, 6), (S7, A7, s7, H, 7), (S8, A8, s8, I, 8), (S9, A9, s9, J, 9), (S10, A10, s10, K, 10), (S11, A11, s11, L, 11), (S12, A12, s12, M, 12), (S13, A13, s13, N, 13), (S14, A14, s14, O, 14));
sum!(Sum16; (S0, A0, s0, A, 0), (S1, A1, s1, B, 1), (S2, A2, s2, C, 2), (S3, A3, s3, D, 3), (S4, A4, s4, E, 4), (S5, A5, s5, F, 5), (S6, A6, s6, G, 6), (S7, A7, s7, H, 7), (S8, A8, s8, I, 8), (S9, A9, s9, J, 9), (S10, A10, s10, K, 10), (S11, A11, s11, L, 11), (S12, A12, s12, M, 12), (S13, A13, s13, N, 13), (S14, A14, s14, O, 14), (S15, A15, s15, P, 15));
