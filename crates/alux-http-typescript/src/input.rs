//! States where each argument comes from, in the vocabulary a caller writes.

use alux_shape::ShapeOf;
use alux_shape_typescript::{TsShape, TsType};
use core::marker::PhantomData;

/// Where a call says one argument goes.
///
/// A caller states an argument once; what it becomes on the wire is what the declaration said, so
/// the generated module carries the role rather than a hand-written request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsSource {
    /// A segment the path binds.
    Path,
    /// A value in the query string.
    Query,
    /// A header the caller sends.
    Header,
    /// A cookie the caller sends.
    Cookie,
    /// The request body, written as a document.
    Body,
    /// The request body, written as a form.
    Form,
    /// The request body, written as parts.
    Multipart,
    /// The request body, sent as it stands.
    Raw,
    /// Something a caller does not state.
    Unstated,
}

impl TsSource {
    /// Returns the word a generated module carries this role as.
    pub fn label(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Query => "query",
            Self::Header => "header",
            Self::Cookie => "cookie",
            Self::Body => "body",
            Self::Form => "form",
            Self::Multipart => "multipart",
            Self::Raw => "raw",
            Self::Unstated => "none",
        }
    }
}

/// One argument, as a call takes it.
#[derive(Debug, Clone)]
pub struct TsArgument {
    /// Where the argument goes.
    pub source: TsSource,
    /// The type the argument carries.
    pub shape: TsType,
}

/// States how a call takes one argument.
pub trait TsInputAlg {
    /// Reads the type this argument carries, and where it goes.
    fn describe(shapes: &TsShape) -> TsArgument;
}

macro_rules! ts_inputs {
    ($($marker:ident => $source:expr, $meaning:literal),+ $(,)?) => {
        $(
            #[doc = concat!("Takes ", $meaning, ".")]
            pub struct $marker<Input>(PhantomData<Input>);

            impl<Input> TsInputAlg for $marker<Input>
            where
                Input: ShapeOf<TsShape, Shape = TsType>,
            {
                fn describe(shapes: &TsShape) -> TsArgument {
                    TsArgument { source: $source, shape: Input::shape_of(shapes) }
                }
            }
        )+
    };
}

ts_inputs! {
    TsPathInput      => TsSource::Path, "a segment the path binds",
    TsQueryInput     => TsSource::Query, "a value in the query string",
    TsHeaderInput    => TsSource::Header, "a header the caller sends",
    TsCookieInput    => TsSource::Cookie, "a cookie the caller sends",
    TsBodyInput      => TsSource::Body, "a request body written as a document",
    TsFormInput      => TsSource::Form, "a request body written as a form",
    TsMultipartInput => TsSource::Multipart, "a request body written as parts",
    TsRawBodyInput   => TsSource::Raw, "a request body sent as it stands",
}

/// Takes nothing, because a caller states nothing here.
pub struct TsUnstatedInput<Input>(PhantomData<Input>);

impl<Input> TsInputAlg for TsUnstatedInput<Input>
where
    Input: ShapeOf<TsShape, Shape = TsType>,
{
    fn describe(shapes: &TsShape) -> TsArgument {
        TsArgument { source: TsSource::Unstated, shape: Input::shape_of(shapes) }
    }
}

/// States the whole argument product a call takes, in declaration order.
pub trait TsInputsAlg {
    /// Reads each argument, in the order the declaration states them.
    fn describe(shapes: &TsShape) -> Vec<TsArgument>;
}

impl TsInputsAlg for () {
    fn describe(_shapes: &TsShape) -> Vec<TsArgument> {
        Vec::new()
    }
}

macro_rules! ts_products {
    ($($input:ident),+ $(,)?) => {
        impl<$($input),+> TsInputsAlg for ($($input,)+)
        where
            $($input: TsInputAlg,)+
        {
            fn describe(shapes: &TsShape) -> Vec<TsArgument> {
                vec![$($input::describe(shapes),)+]
            }
        }
    };
}

ts_products!(I1);
ts_products!(I1, I2);
ts_products!(I1, I2, I3);
ts_products!(I1, I2, I3, I4);
ts_products!(I1, I2, I3, I4, I5);
ts_products!(I1, I2, I3, I4, I5, I6);
ts_products!(I1, I2, I3, I4, I5, I6, I7);
ts_products!(I1, I2, I3, I4, I5, I6, I7, I8);
ts_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9);
ts_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10);
ts_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11);
ts_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12);
ts_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12, I13);
ts_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12, I13, I14);
ts_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12, I13, I14, I15);
ts_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12, I13, I14, I15, I16);
