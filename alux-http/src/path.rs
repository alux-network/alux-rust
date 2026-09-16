//! States which path a selector matches, independently of how a router spells it.

/// Names one part of a route path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathSegment {
    /// Matches this segment exactly.
    Literal(String),
    /// Binds one segment under this name.
    Param(String),
    /// Binds every remaining segment under this name.
    Tail(String),
}

impl PathSegment {
    /// Returns the name this segment binds, where it binds one.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Literal(_) => None,
            Self::Param(name) | Self::Tail(name) => Some(name),
        }
    }
}

/// Carries a route path as the segments it matches.
///
/// Routers disagree about how a parameter is written, so a path held as a string is a path written
/// for one framework. A path is read into segments once, here, and each interpreter spells those
/// segments the way its own router reads them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct RoutePath {
    segments: Vec<PathSegment>,
}

impl RoutePath {
    /// Reads a path written in any of the spellings the major routers accept.
    ///
    /// `:name` and `{name}` bind one segment, `*name` and `{*name}` bind every remaining segment,
    /// and any other segment is matched literally. Empty segments state nothing, so leading,
    /// trailing, and repeated separators are dropped.
    pub fn parse(path: &str) -> Self {
        let segments = path.split('/').filter(|segment| !segment.is_empty()).map(read_segment).collect();

        Self { segments }
    }

    /// Returns the segments this path matches, in order.
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }

    /// Returns whether this path matches nothing of its own.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Returns the names this path binds, in order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.segments.iter().filter_map(PathSegment::name)
    }
}

impl From<&str> for RoutePath {
    fn from(path: &str) -> Self {
        Self::parse(path)
    }
}

/// Reads one non-empty segment as what it binds.
fn read_segment(segment: &str) -> PathSegment {
    if let Some(name) = segment.strip_prefix(':') {
        return PathSegment::Param(name.to_owned());
    }
    if let Some(name) = segment.strip_prefix('*') {
        return PathSegment::Tail(name.to_owned());
    }
    if let Some(name) = segment.strip_prefix('{').and_then(|segment| segment.strip_suffix('}')) {
        return match name.strip_prefix('*') {
            Some(name) => PathSegment::Tail(name.to_owned()),
            None => PathSegment::Param(name.to_owned()),
        };
    }

    PathSegment::Literal(segment.to_owned())
}

/// Spells route-path parameters the way one router reads them.
pub trait PathSyntaxAlg {
    /// Returns the spelling of a parameter binding one segment.
    fn param(&self, name: &str) -> String;

    /// Returns the spelling of a parameter binding every remaining segment.
    fn tail(&self, name: &str) -> String;
}

/// Spells parameters the way a described surface states them.
///
/// Every interpreter describes its composed surface in this spelling, which is what makes two
/// interpretations of one program comparable. A router is handed its own spelling instead.
#[derive(Debug, Default)]
pub struct CanonicalPath;

impl PathSyntaxAlg for CanonicalPath {
    fn param(&self, name: &str) -> String {
        format!("{{{name}}}")
    }

    fn tail(&self, name: &str) -> String {
        format!("{{*{name}}}")
    }
}

/// Composes one absolute path from the parts a selector holds, spelled as `syntax` reads them.
///
/// Selector composition concatenates path parts, so every interpreter shares one rule: each segment
/// is preceded by exactly one separator, and a selector holding no segment matches the root.
pub fn compose_path<'a, Parts>(parts: Parts, syntax: &impl PathSyntaxAlg) -> String
where
    Parts: IntoIterator<Item = &'a RoutePath>,
{
    let mut path = String::new();
    for part in parts {
        for segment in part.segments() {
            path.push('/');
            match segment {
                PathSegment::Literal(value) => path.push_str(value),
                PathSegment::Param(name) => path.push_str(&syntax.param(name)),
                PathSegment::Tail(name) => path.push_str(&syntax.tail(name)),
            }
        }
    }

    if path.is_empty() {
        path.push('/');
    }

    path
}

/// Describes one absolute path in the spelling every interpretation states it in.
pub fn describe_path<'a, Parts>(parts: Parts) -> String
where
    Parts: IntoIterator<Item = &'a RoutePath>,
{
    compose_path(parts, &CanonicalPath)
}

#[cfg(test)]
mod tests {
    use super::{PathSegment, RoutePath, describe_path};

    #[test]
    fn reads_every_spelling_a_router_accepts_as_the_same_path() {
        let poem = RoutePath::parse("/status/:id/*rest");
        let axum = RoutePath::parse("/status/{id}/{*rest}");

        assert_eq!(poem, axum);
        assert_eq!(
            poem.segments(),
            [
                PathSegment::Literal("status".to_owned()),
                PathSegment::Param("id".to_owned()),
                PathSegment::Tail("rest".to_owned()),
            ]
        );
        assert_eq!(poem.names().collect::<Vec<_>>(), ["id", "rest"]);
    }

    #[test]
    fn states_nothing_for_a_separator_that_states_nothing() {
        assert!(RoutePath::parse("/").is_empty());
        assert_eq!(RoutePath::parse("//status//"), RoutePath::parse("status"));
        assert_eq!(describe_path([&RoutePath::parse("/")]), "/");
    }

    #[test]
    fn composes_parts_into_one_absolute_path() {
        let parts = [RoutePath::parse("/api"), RoutePath::parse("v1"), RoutePath::parse("/status/:id")];

        assert_eq!(describe_path(&parts), "/api/v1/status/{id}");
    }

    #[test]
    fn matches_a_segment_no_router_reads_as_a_parameter_literally() {
        assert_eq!(RoutePath::parse("{id}.json").segments(), [PathSegment::Literal("{id}.json".to_owned())]);
    }
}
