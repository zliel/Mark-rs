use std::{
    mem::take,
    path::{Path, PathBuf},
};

/// Utility function for pushing a String buffer to a generic collection.
///
/// This function checks if the buffer is not empty, converts it to the type `T`, and pushes it to
/// the collection.
///
/// # Type Parameters
/// * `T` - The type to which the buffer will be converted. It must implement the
///   `From<String>` trait.
///
/// # Arguments
/// * `collection` - A mutable reference to the collection of type `Vec<T>` where the buffer will be pushed.
/// * `buffer` - A mutable reference to the String buffer that will be converted and pushed.
pub fn push_buffer_to_collection<T>(collection: &mut Vec<T>, buffer: &mut String)
where
    T: From<String>,
{
    if !buffer.is_empty() {
        collection.push(T::from(take(buffer)));
    }
}

/// Builds a relative prefix path based on the provided HTML relative path.
/// Note that this function does not add the final file name, only the preceding directories.
pub fn build_rel_prefix(html_rel_path: &str) -> PathBuf {
    let rel_path = Path::new(html_rel_path);
    let depth = rel_path.parent().map_or(0, |p| p.components().count());
    let mut rel_prefix = PathBuf::new();
    for _ in 0..depth {
        rel_prefix.push("..");
    }
    rel_prefix
}
