//! States where each argument comes from, in the vocabulary a document reads.

use alux_shape::ShapeOf;
use alux_shape_jsonschema::{JsonSchema, JsonSchemaShape};
use core::marker::PhantomData;
use serde_json::Value;

/// Where a document says one argument comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenApiSource {
    /// A segment the path binds.
    Path,
    /// A value in the query string.
    Query,
    /// A header the caller sent.
    Header,
    /// A cookie the caller sent.
    Cookie,
    /// The request body, read as the stated media type.
    Body(&'static str),
    /// Something the caller never states, which a document therefore does not describe.
    Unstated,
}

/// One argument, as a document describes it.
#[derive(Debug, Clone)]
pub struct OpenApiArgument {
    /// Where the argument comes from.
    pub source: OpenApiSource,
    /// What the argument states there.
    pub stated: OpenApiStated,
}

/// What one argument states where it comes from.
///
/// A body is one value, so an argument read from it states one schema. A query string, a header
/// collection, and a cookie collection are names and values, so an argument read from one of those
/// states a member for each name, which is what a document keys a parameter by.
#[derive(Debug, Clone)]
pub enum OpenApiStated {
    /// One value, which is what a body carries.
    Whole(Value),
    /// One value per name, which is what a collection of names and values carries.
    Named(Vec<OpenApiNamed>),
}

/// One named value an argument is read from.
#[derive(Debug, Clone)]
pub struct OpenApiNamed {
    /// The name this value is stated under.
    pub name: String,
    /// The schema this value carries.
    pub schema: Value,
    /// Whether a caller must state it.
    pub required: bool,
}

/// Reads what a shape states as the named values a collection carries.
///
/// A product read from names and values is one parameter per member, not one parameter carrying the
/// product. Anything else is stated whole, which is the only thing a document can say about it.
fn named_values(schema: &JsonSchemaShape, stated: Value) -> OpenApiStated {
    let resolved = resolved(schema, &stated);
    let Some(object) = resolved.as_object() else {
        return OpenApiStated::Whole(stated);
    };
    if object.get("type").and_then(Value::as_str) != Some("object") {
        return OpenApiStated::Whole(stated);
    }
    let Some(properties) = object.get("properties").and_then(Value::as_object) else {
        return OpenApiStated::Whole(stated);
    };
    let required = object.get("required").and_then(Value::as_array).cloned().unwrap_or_default();
    let named = properties
        .iter()
        .map(|(name, schema)| OpenApiNamed {
            name: name.clone(),
            schema: schema.clone(),
            required: required.iter().any(|stated| stated.as_str() == Some(name)),
        })
        .collect();

    OpenApiStated::Named(named)
}

/// Returns the shape a reference names, or the shape itself where it names none.
fn resolved(schema: &JsonSchemaShape, stated: &Value) -> Value {
    let Some(reference) = stated.get("$ref").and_then(Value::as_str) else {
        return stated.clone();
    };
    let named = reference.rsplit('/').next().unwrap_or_default();

    schema.definitions().get(named).cloned().unwrap_or_else(|| stated.clone())
}

macro_rules! openapi_inputs {
    ($($marker:ident => $source:expr, $stated:ident, $meaning:literal),+ $(,)?) => {
        $(
            #[doc = concat!("Describes ", $meaning, " in a document.")]
            pub struct $marker<Input>(PhantomData<Input>);

            impl<Input> OpenApiInputAlg for $marker<Input>
            where
                Input: ShapeOf<JsonSchemaShape, Shape = JsonSchema>,
            {
                fn describe(schema: &JsonSchemaShape) -> OpenApiArgument {
                    let stated = Input::shape_of(schema).into_value();

                    OpenApiArgument { source: $source, stated: $stated(schema, stated) }
                }
            }
        )+
    };
}

/// Reads a shape stated whole, which is what a body carries.
fn whole(_schema: &JsonSchemaShape, stated: Value) -> OpenApiStated {
    OpenApiStated::Whole(stated)
}

/// States how a document describes one argument.
pub trait OpenApiInputAlg {
    /// Describes this argument, naming the shapes it states along the way.
    fn describe(schema: &JsonSchemaShape) -> OpenApiArgument;
}

openapi_inputs! {
    OpenApiPathInput      => OpenApiSource::Path, whole, "a segment the path binds",
    OpenApiQueryInput     => OpenApiSource::Query, named_values, "a value in the query string",
    OpenApiHeaderInput    => OpenApiSource::Header, named_values, "a header the caller sent",
    OpenApiCookieInput    => OpenApiSource::Cookie, named_values, "a cookie the caller sent",
    OpenApiBodyInput      => OpenApiSource::Body("application/json"), whole, "a request body read as a document",
    OpenApiFormInput      => OpenApiSource::Body("application/x-www-form-urlencoded"), whole, "a form-encoded request body",
    OpenApiMultipartInput => OpenApiSource::Body("multipart/form-data"), whole, "a request body arriving as parts",
    OpenApiRawBodyInput   => OpenApiSource::Body("application/octet-stream"), whole, "a request body taken as it arrived",
}

/// Describes an argument a caller never states.
///
/// An endpoint context is how a request reaches a handler, not something a caller sends, so a
/// document describes no parameter for it. The shape is still read, because the argument still has
/// one.
pub struct OpenApiUnstatedInput<Input>(PhantomData<Input>);

impl<Input> OpenApiInputAlg for OpenApiUnstatedInput<Input> {
    fn describe(_schema: &JsonSchemaShape) -> OpenApiArgument {
        OpenApiArgument { source: OpenApiSource::Unstated, stated: OpenApiStated::Whole(Value::Null) }
    }
}

/// Describes the whole argument product one endpoint states, in declaration order.
pub trait OpenApiInputsAlg {
    /// Describes each argument, in the order the declaration reads them.
    fn describe(schema: &JsonSchemaShape) -> Vec<OpenApiArgument>;
}

impl OpenApiInputsAlg for () {
    fn describe(_schema: &JsonSchemaShape) -> Vec<OpenApiArgument> {
        Vec::new()
    }
}

macro_rules! openapi_products {
    ($($input:ident),+ $(,)?) => {
        impl<$($input),+> OpenApiInputsAlg for ($($input,)+)
        where
            $($input: OpenApiInputAlg,)+
        {
            fn describe(schema: &JsonSchemaShape) -> Vec<OpenApiArgument> {
                vec![$($input::describe(schema),)+]
            }
        }
    };
}

openapi_products!(I1);
openapi_products!(I1, I2);
openapi_products!(I1, I2, I3);
openapi_products!(I1, I2, I3, I4);
openapi_products!(I1, I2, I3, I4, I5);
openapi_products!(I1, I2, I3, I4, I5, I6);
openapi_products!(I1, I2, I3, I4, I5, I6, I7);
openapi_products!(I1, I2, I3, I4, I5, I6, I7, I8);
openapi_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9);
openapi_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10);
openapi_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11);
openapi_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12);
openapi_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12, I13);
openapi_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12, I13, I14);
openapi_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12, I13, I14, I15);
openapi_products!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12, I13, I14, I15, I16);
