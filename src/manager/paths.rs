use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathResolveResult {
    Ok(PathBuf),
    NoDataAncestor,
    InvalidPath,
}

/// Resolves a zip entry path to a Data/-relative path for DAVA asset deployment.
pub fn resolve_data_relative_path(name_in_zip: &str) -> PathResolveResult {
    let normalized = name_in_zip.replace('\\', "/");
    if normalized.contains("..") {
        return PathResolveResult::InvalidPath;
    }

    let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();

    let mut last_data_pos = None;
    for (i, part) in parts.iter().enumerate() {
        if *part == "Data" {
            last_data_pos = Some(i);
        }
    }

    match last_data_pos {
        Some(pos) if pos + 1 < parts.len() => {
            PathResolveResult::Ok(PathBuf::from(parts[(pos + 1)..].join("/")))
        }
        _ => PathResolveResult::NoDataAncestor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_author_wrapper() {
        match resolve_data_relative_path("AuthorPack/Data/Gfx/tex.pvr.dvpl") {
            PathResolveResult::Ok(p) => assert_eq!(p.to_string_lossy(), "Gfx/tex.pvr.dvpl"),
            _ => panic!("expected Ok"),
        }
    }

    #[test]
    fn handles_double_data_wrap() {
        match resolve_data_relative_path("Data/Data/XML/list.xml.dvpl") {
            PathResolveResult::Ok(p) => assert_eq!(p.to_string_lossy(), "XML/list.xml.dvpl"),
            _ => panic!("expected Ok"),
        }
    }

    #[test]
    fn rejects_no_data_ancestor() {
        assert_eq!(
            resolve_data_relative_path("Gfx/tex.pvr.dvpl"),
            PathResolveResult::NoDataAncestor
        );
    }

    #[test]
    fn rejects_parent_traversal() {
        assert_eq!(
            resolve_data_relative_path("Data/../etc/passwd"),
            PathResolveResult::InvalidPath
        );
    }
}
