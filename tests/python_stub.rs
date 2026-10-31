#![cfg(not(target_arch = "wasm32"))]

//! The Python stub's hints for the float vector and integer list readers.
//! The build writes the stub, so these tests read what the generator just made.

use std::collections::BTreeMap;
use std::fs;

const STUB: &str = "platforms/python/fragmentcolor/fragmentcolor.pyi";

/// The ndarray every float vector reader takes: no uint8 bytes form and no bool array.
const FLOAT_ARRAY: &str = "npt.NDArray[numpy.floating[Any] | numpy.signedinteger[Any] | numpy.uint16 | numpy.uint32 | numpy.uint64]";

/// The parameter hints of each `Class.method`, with the stub's `_Name` aliases filled in.
fn read_hints() -> BTreeMap<String, BTreeMap<String, String>> {
    let stub = fs::read_to_string(STUB).expect("the build writes the Python stub");
    let aliases: BTreeMap<&str, &str> = stub
        .lines()
        .filter_map(|line| line.split_once(": TypeAlias = "))
        .collect();
    let mut hints = BTreeMap::new();
    let mut class = "";
    for line in stub.lines() {
        if let Some(rest) = line.strip_prefix("class ") {
            class = rest.split([':', '(']).next().unwrap_or_default();
            continue;
        }
        let Some(rest) = line.trim_start().strip_prefix("def ") else {
            continue;
        };
        let Some((method, rest)) = rest.split_once('(') else {
            continue;
        };
        let Some((arguments, _)) = rest.rsplit_once(") ->") else {
            continue;
        };
        let parameters = split_top_level(arguments)
            .into_iter()
            .filter_map(|parameter| parameter.split_once(": "))
            .map(|(name, hint)| {
                let hint = hint.trim_end_matches(" = ...");
                let hint = hint
                    .split(" | ")
                    .map(|part| aliases.get(part).copied().unwrap_or(part))
                    .collect::<Vec<_>>()
                    .join(" | ");
                (name.to_string(), hint)
            })
            .collect();
        hints.insert(format!("{class}.{method}"), parameters);
    }
    hints
}

/// The parts of a parameter list between its top-level commas.
fn split_top_level(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let (mut depth, mut start) = (0usize, 0usize);
    for (index, letter) in text.char_indices() {
        match letter {
            '[' | '(' => depth += 1,
            ']' | ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(text[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(text[start..].trim());
    parts
}

/// The hint of `parameter` in `method`, such as `Light.point`.
fn hint<'a>(
    hints: &'a BTreeMap<String, BTreeMap<String, String>>,
    method: &str,
    parameter: &str,
) -> &'a str {
    hints
        .get(method)
        .and_then(|parameters| parameters.get(parameter))
        .map(String::as_str)
        .unwrap_or_else(|| panic!("the stub has no {method}({parameter})"))
}

// Story: each float vector parameter names the ndarray its reader takes, so pyright turns
// away the uint8 and bool arrays the reader refuses. The hint never falls back to
// `npt.NDArray[Any]`, or to a bare `SupportsFloat`, which every ndarray satisfies.
#[test]
fn float_vector_hints_name_their_ndarray() {
    let hints = read_hints();
    let readers = [
        ("Pass.set_clear_color", "color"),
        ("Light.point", "position"),
        ("Light.point", "color"),
        ("Light.spot", "direction"),
        ("Light.set_position", "position"),
        ("Light.set_color", "color"),
        ("Scene.ambient", "color"),
        ("Model.translate", "offset"),
        ("Model.scale", "factor"),
        ("Model.rotate", "axis"),
        ("Model.set_transform", "matrix"),
        ("Camera.look_at", "up"),
        ("Material.base_color", "color"),
        ("Material.uv_transform", "offset"),
        ("Material.texture_transform", "scale"),
        ("Quad.__new__", "min"),
        ("Quad.__new__", "max"),
        ("Vertex.__new__", "position"),
        ("Vertex.pbr", "position"),
        // The position, uv and color of a vertex tuple.
        ("Mesh.add_vertex", "vertex"),
        ("Mesh.add_vertices", "vertices"),
        ("Mesh.from_vertices", "vertices"),
    ];
    for (method, parameter) in readers {
        let hint = hint(&hints, method, parameter);
        assert!(hint.contains(FLOAT_ARRAY), "{method}({parameter}): {hint}");
        assert!(
            !hint.contains("NDArray[Any]"),
            "{method}({parameter}): {hint}"
        );
        let outside_lists = hint.replace("Sequence[SupportsFloat]", "");
        assert!(
            !outside_lists.contains("SupportsFloat"),
            "{method}({parameter}): {hint}"
        );
    }
    let position = hint(&hints, "Vertex.__new__", "position");
    assert!(position.contains("float | numpy.floating[Any] | numpy.integer[Any]"));
}

// Story: a list of ints takes an integer ndarray and nothing wider, and a size's fields
// come in any mapping, which the runtime reads as it reads a dict.
#[test]
fn integer_hints_take_integer_arrays_and_mappings() {
    let hints = read_hints();
    let indices = hint(&hints, "Mesh.set_indices", "indices");
    assert_eq!(
        indices,
        "Sequence[SupportsIndex] | npt.NDArray[numpy.integer[Any]]"
    );
    let size = hint(&hints, "TextureTarget.resize", "size");
    assert!(size.contains("Mapping[str, SupportsIndex]"), "{size}");
    assert!(size.contains("npt.NDArray[numpy.integer[Any]]"), "{size}");
    assert!(!size.contains("NDArray[Any]"), "{size}");
}
