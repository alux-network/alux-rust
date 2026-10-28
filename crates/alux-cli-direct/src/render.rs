use std::{ffi::OsString, path::PathBuf};

/// Renders a command's successful value as the text it writes to standard output.
///
/// A domain type renders itself by implementing this. A value that renders nothing writes nothing.
pub trait DirectRender {
    /// Appends this value's text to `out`, without a trailing newline.
    fn render(&self, out: &mut String);
}

impl DirectRender for () {
    fn render(&self, _: &mut String) {}
}

macro_rules! displayed {
    ($($value:ty),* $(,)?) => {
        $(impl DirectRender for $value {
            fn render(&self, out: &mut String) {
                out.push_str(&self.to_string());
            }
        })*
    };
}

displayed!(String, &str, char, bool, f32, f64, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

impl DirectRender for PathBuf {
    fn render(&self, out: &mut String) {
        out.push_str(&self.display().to_string());
    }
}

impl DirectRender for OsString {
    fn render(&self, out: &mut String) {
        out.push_str(&self.to_string_lossy());
    }
}

/// Renders nothing when absent.
impl<Value: DirectRender> DirectRender for Option<Value> {
    fn render(&self, out: &mut String) {
        if let Some(value) = self {
            value.render(out);
        }
    }
}

/// Renders one value per line.
impl<Value: DirectRender> DirectRender for Vec<Value> {
    fn render(&self, out: &mut String) {
        for (index, value) in self.iter().enumerate() {
            if index > 0 {
                out.push('\n');
            }
            value.render(out);
        }
    }
}
