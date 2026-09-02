//! Renders templates/FindCrate.cmake.tmpl by substituting @VAR@ placeholders.

const TEMPLATE: &str = include_str!("../templates/FindCrate.cmake.tmpl");

pub fn to_pascal_case(name: &str) -> String {
    name.split(['_', '-'])
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

pub fn render(info: &crate::metadata::CrateInfo) -> String {
    let name_pascal = to_pascal_case(&info.name);
    let name_upper = info.name.to_uppercase().replace('-', "_");

    let header_dir = info
        .header_paths
        .first()
        .and_then(|p| p.parent())
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let header_name = info
        .header_paths
        .first()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let lib_dir = info
        .lib_path
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let lib_name = info
        .lib_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let link_type = match info.lib_kind {
        crate::metadata::LibKind::Static => "STATIC",
        crate::metadata::LibKind::Shared => "SHARED",
    };
    let implib_path = info.implib_path.as_deref().map(path_string).unwrap_or_default();
    let lib_path_debug = info.lib_path_debug.as_deref().map(path_string).unwrap_or_default();
    let lib_path_release = info.lib_path_release.as_deref().map(path_string).unwrap_or_default();

    let extra_link_libs = info.link_libs.join(" ");

    TEMPLATE
        .replace("@CRATE_NAME_PASCAL@", &name_pascal)
        .replace("@CRATE_NAME_UPPER@", &name_upper)
        .replace("@CRATE_VERSION@", &info.version)
        .replace("@HEADER_NAME@", &header_name)
        .replace("@HEADER_DIR@", &header_dir)
        .replace("@LIB_NAME@", &lib_name)
        .replace("@LIB_DIR@", &lib_dir)
        .replace("@LIB_LINK_TYPE@", link_type)
        .replace("@IMPLIB_PATH@", &implib_path)
        .replace("@LIB_PATH_DEBUG@", &lib_path_debug)
        .replace("@LIB_PATH_RELEASE@", &lib_path_release)
        .replace("@EXTRA_LINK_LIBS@", &extra_link_libs)
}

fn path_string(p: &std::path::Path) -> String {
    p.display().to_string()
}

