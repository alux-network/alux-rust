/// Appends one normalized path segment to a composed absolute path.
///
/// Selector composition concatenates path parts, so interpreters share one normalization rule:
/// leading and trailing slashes are removed from `part`, exactly one separator is inserted before a
/// non-empty segment, and empty or root-only parts leave `path` unchanged.
pub fn append_path(path: &mut String, part: &str) {
    let part = part.trim_matches('/');
    if !part.is_empty() {
        path.push('/');
        path.push_str(part);
    }
}
