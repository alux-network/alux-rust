//! Mappings from the cases of one type to values of another.
//!
//! A case is a value, so this is a mapping between values. What makes it worth stating as one is
//! that the left side is one of finitely many cases: that is what makes the list of them complete,
//! and what makes reading a case back from its value possible at all.
//!
//! A mapping is stated as a module of plain functions rather than as an inherent impl, so it may
//! be stated for a type another crate owns: what a value *is* belongs to whoever defined it, and
//! what it maps to belongs to whoever needs the mapping.
//!
//! The module is named after the type, so a mapping states one thing — what each case maps to.
//! Inside it nothing needs a name of its own: `to` states the value, `from` reads the case back,
//! and `ALL` states every case.

/// Maps every case of one type to a value, reads the case back from it, and states them all.
///
/// The module is named after the type it maps, so the mapping repeats no name that was already
/// given. Inside it, `to` states the value, `from` reads the case back, and `ALL` states every
/// case.
///
/// The forward direction is a `match` with no wildcard, so the compiler states that the cases
/// named here are every case there is: that is what makes `ALL` complete, because leaving a case
/// out does not compile. `from` stays partial, because a value naming no case is something a
/// reader can write.
///
/// `to` returns the value type, and `from` takes the type written after `as`. The two are written
/// separately because they are often not the same type: in `&'static str as &str`, `to` returns a
/// word baked into the program, while `from` accepts any word, including one read at run time —
/// which would not compile if `from` demanded a `&'static str`. Where the two coincide, as they do
/// for `u8` or `char`, the `as` clause is left off.
///
/// ```
/// use alux_sdk::case_mapping;
///
/// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// pub enum Direction {
///     North,
///     South,
/// }
///
/// case_mapping! {
///     Direction, &'static str as &str,
///         North <=> "north",
///         South <=> "south",
/// }
///
/// fn main() {
///     assert_eq!(direction::to(Direction::North), "north");
///     assert_eq!(direction::from("south"), Some(Direction::South));
///     assert_eq!(direction::from("sideways"), None);
///     assert_eq!(direction::ALL, &[Direction::North, Direction::South]);
/// }
/// ```
#[macro_export]
macro_rules! case_mapping {
    (
        $ty:ident,
        $value:ty as $read:ty,
        $($variant:ident <=> $stated:expr),* $(,)?
    ) => {
        $crate::paste::paste! {
            #[doc = concat!("States what each `", stringify!($ty), "` maps to, and reads it back.")]
            pub mod [<$ty:snake>] {
                /// States every case this mapping covers, in the order they are stated.
                pub const ALL: &[super::$ty] = &[$(super::$ty::$variant),*];

                /// States the value one case maps to.
                #[must_use]
                pub const fn to(case: super::$ty) -> $value {
                    match case {
                        $(super::$ty::$variant => $stated,)*
                    }
                }

                /// Reads back the case one value names, when it names one.
                #[must_use]
                pub fn from(value: $read) -> Option<super::$ty> {
                    $(
                        if value == $stated {
                            return Some(super::$ty::$variant);
                        }
                    )*
                    None
                }
            }
        }
    };

    // Same-type form, for values a reader states exactly as a case maps to them.
    (
        $ty:ident,
        $value:ty,
        $($variant:ident <=> $stated:expr),* $(,)?
    ) => {
        $crate::case_mapping! {
            $ty, $value as $value,
            $($variant <=> $stated),*
        }
    };
}

/// Maps some cases of one type to values, reads those cases back, and states the ones it maps.
///
/// The module is stated exactly as [`case_mapping`] states it. Both directions are partial
/// here: a case the mapping does not name maps to nothing, and a value naming no case names
/// nothing. `ALL` states the cases the mapping maps, which is not every case the type has.
///
/// ```
/// use alux_sdk::case_mapping_partial;
///
/// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// pub enum Direction {
///     North,
///     South,
///     Nowhere,
/// }
///
/// case_mapping_partial! {
///     Direction, &'static str as &str,
///         North <=> "north",
///         South <=> "south",
/// }
///
/// fn main() {
///     assert_eq!(direction::to(Direction::North), Some("north"));
///     assert_eq!(direction::to(Direction::Nowhere), None);
///     assert_eq!(direction::from("south"), Some(Direction::South));
///     assert_eq!(direction::ALL, &[Direction::North, Direction::South]);
/// }
/// ```
#[macro_export]
macro_rules! case_mapping_partial {
    (
        $ty:ident,
        $value:ty as $read:ty,
        $($variant:ident <=> $stated:expr),* $(,)?
    ) => {
        $crate::paste::paste! {
            #[doc = concat!("States what some `", stringify!($ty), "` cases map to, and reads them back.")]
            pub mod [<$ty:snake>] {
                /// States every case this mapping maps, in the order they are stated.
                pub const ALL: &[super::$ty] = &[$(super::$ty::$variant),*];

                /// States the value one case maps to, when it maps to one.
                #[must_use]
                #[allow(unreachable_patterns)]
                pub const fn to(case: super::$ty) -> Option<$value> {
                    match case {
                        $(super::$ty::$variant => Some($stated),)*
                        _ => None,
                    }
                }

                /// Reads back the case one value names, when it names one.
                #[must_use]
                pub fn from(value: $read) -> Option<super::$ty> {
                    $(
                        if value == $stated {
                            return Some(super::$ty::$variant);
                        }
                    )*
                    None
                }
            }
        }
    };

    // Same-type form, for values a reader states exactly as a case maps to them.
    (
        $ty:ident,
        $value:ty,
        $($variant:ident <=> $stated:expr),* $(,)?
    ) => {
        $crate::case_mapping_partial! {
            $ty, $value as $value,
            $($variant <=> $stated),*
        }
    };
}

#[cfg(test)]
mod tests {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum SoundLevel {
        Loud,
        Quiet,
        Unstated,
    }

    case_mapping! {
        SoundLevel, &'static str as &str,
            Loud     <=> "loud",
            Quiet    <=> "quiet",
            Unstated <=> "unstated",
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Loudness {
        Loud,
        Quiet,
        Unstated,
    }

    case_mapping_partial! {
        Loudness, u8,
            Loud  <=> 2,
            Quiet <=> 1,
    }

    #[test]
    fn a_mapping_is_named_after_the_type_it_maps() {
        assert_eq!(sound_level::to(SoundLevel::Loud), "loud");
        assert_eq!(sound_level::from("quiet"), Some(SoundLevel::Quiet));
        assert_eq!(sound_level::from("nothing anybody would call a sound"), None);
        assert_eq!(sound_level::ALL, &[SoundLevel::Loud, SoundLevel::Quiet, SoundLevel::Unstated]);
    }

    #[test]
    fn a_partial_mapping_states_nothing_for_a_case_it_does_not_name() {
        assert_eq!(loudness::to(Loudness::Loud), Some(2));
        assert_eq!(loudness::to(Loudness::Unstated), None);
        assert_eq!(loudness::from(1), Some(Loudness::Quiet));
        assert_eq!(loudness::ALL, &[Loudness::Loud, Loudness::Quiet]);
    }
}
