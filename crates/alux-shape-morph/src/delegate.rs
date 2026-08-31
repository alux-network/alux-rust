//! Forwarding the primitives a transformation does not change.
//!
//! A transformation states the few operations it means to alter, and everything else reaches the
//! algebra beneath it untouched. Writing that out per transformation would bury the one line that
//! matters, so it is stated once here — in the two arrangements a transformation actually takes: one
//! that states its own `named`, and one that does not.

/// Forwards every shape primitive but `named`, which the transformation states itself.
macro_rules! delegate_shape_except_named {
    ($morph:ident) => {
        impl<A> ::alux_shape::Sorts for $morph<A>
        where
            A: ::alux_shape::Sorts,
        {
            type Ty = A::Ty;
            type Field = A::Field;
        }

        impl<A> ::alux_shape::ShapeAlg for $morph<A>
        where
            A: ::alux_shape::ShapeAlg,
        {
            $crate::delegate::forwarded_but_named!();
        }
    };
}

/// The body of every forwarded shape primitive except `named`.
macro_rules! forwarded_but_named {
    () => {
        fn truth(&self) -> A::Ty {
            self.inner().truth()
        }

        fn unit(&self) -> A::Ty {
            self.inner().unit()
        }

        fn text(&self) -> A::Ty {
            self.inner().text()
        }

        fn literal(&self, text: &str) -> A::Ty {
            self.inner().literal(text)
        }

        fn name_word(&self, words: ::alux_shape::Words<'_>) -> A::Ty {
            self.inner().name_word(words)
        }

        fn int(&self, signed: bool, bits: u16) -> A::Ty {
            self.inner().int(signed, bits)
        }

        fn float(&self, bits: u16) -> A::Ty {
            self.inner().float(bits)
        }

        fn bytes(&self, len: Option<usize>) -> A::Ty {
            self.inner().bytes(len)
        }

        fn hex(&self, item: A::Ty) -> A::Ty {
            self.inner().hex(item)
        }

        fn decimal(&self, item: A::Ty) -> A::Ty {
            self.inner().decimal(item)
        }

        fn base64(&self, item: A::Ty) -> A::Ty {
            self.inner().base64(item)
        }

        fn opt(&self, item: A::Ty) -> A::Ty {
            self.inner().opt(item)
        }

        fn seq(&self, item: A::Ty) -> A::Ty {
            self.inner().seq(item)
        }

        fn map(&self, key: A::Ty, value: A::Ty) -> A::Ty {
            self.inner().map(key, value)
        }

        fn product(&self, fields: Vec<A::Field>) -> A::Ty {
            self.inner().product(fields)
        }

        fn choice(&self, alternatives: Vec<A::Ty>) -> A::Ty {
            self.inner().choice(alternatives)
        }

        fn named(&self, words: ::alux_shape::Words<'_>, body: A::Ty) -> A::Ty {
            self.rename(words, body)
        }

        fn reference(&self, words: ::alux_shape::Words<'_>) -> A::Ty {
            self.inner().reference(words)
        }
    };
}

/// Forwards both member operations, for a transformation that alters no member.
macro_rules! delegate_members {
    ($morph:ident) => {
        impl<A> ::alux_shape::FieldAlg for $morph<A>
        where
            A: ::alux_shape::ShapeAlg + ::alux_shape::FieldAlg,
        {
            fn field(&self, words: ::alux_shape::Words<'_>, shape: A::Ty) -> A::Field {
                self.inner().field(words, shape)
            }

            fn merge(&self, shape: A::Ty) -> A::Field {
                self.inner().merge(shape)
            }
        }
    };
}

pub(crate) use {delegate_members, delegate_shape_except_named, forwarded_but_named};
