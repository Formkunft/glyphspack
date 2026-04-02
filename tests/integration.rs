use glyphspack::{pack, plist, unpack};
use std::fs;
use std::path::Path;

fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

// Pack tests

#[test]
fn pack_creates_expected_structure() {
    let input = fixture_path("Andika Mini.glyphs");
    let reference = fixture_path("Andika Mini.glyphspackage");
    let output = std::env::temp_dir().join("pack_structure_test.glyphspackage");
    let _ = fs::remove_dir_all(&output);

    pack::pack(&input, &output, true).unwrap();

    // Same set of top-level files
    for name in ["fontinfo.plist", "order.plist", "UIState.plist"] {
        assert!(
            output.join(name).is_file(),
            "{name} missing from packed output"
        );
        assert!(
            reference.join(name).is_file(),
            "{name} missing from reference"
        );
    }
    assert!(output.join("glyphs").is_dir());

    let _ = fs::remove_dir_all(&output);
}

#[test]
fn pack_produces_same_glyph_files_as_reference() {
    let input = fixture_path("Andika Mini.glyphs");
    let reference = fixture_path("Andika Mini.glyphspackage");
    let output = std::env::temp_dir().join("pack_glyphs_test.glyphspackage");
    let _ = fs::remove_dir_all(&output);

    pack::pack(&input, &output, true).unwrap();

    let mut output_glyphs = list_glyph_filenames(&output.join("glyphs"));
    let mut reference_glyphs = list_glyph_filenames(&reference.join("glyphs"));
    output_glyphs.sort();
    reference_glyphs.sort();

    assert_eq!(output_glyphs, reference_glyphs);

    let _ = fs::remove_dir_all(&output);
}

#[test]
fn pack_fontinfo_matches_reference() {
    let input = fixture_path("Andika Mini.glyphs");
    let reference = fixture_path("Andika Mini.glyphspackage");
    let output = std::env::temp_dir().join("pack_fontinfo_test.glyphspackage");
    let _ = fs::remove_dir_all(&output);

    pack::pack(&input, &output, true).unwrap();

    let output_keys = dict_keys_from_file(&output.join("fontinfo.plist"));
    let reference_keys = dict_keys_from_file(&reference.join("fontinfo.plist"));
    assert_eq!(output_keys, reference_keys);

    let _ = fs::remove_dir_all(&output);
}

#[test]
fn pack_order_matches_reference() {
    let input = fixture_path("Andika Mini.glyphs");
    let reference = fixture_path("Andika Mini.glyphspackage");
    let output = std::env::temp_dir().join("pack_order_test.glyphspackage");
    let _ = fs::remove_dir_all(&output);

    pack::pack(&input, &output, true).unwrap();

    let output_order = fs::read_to_string(output.join("order.plist")).unwrap();
    let reference_order = fs::read_to_string(reference.join("order.plist")).unwrap();
    assert_eq!(output_order, reference_order);

    let _ = fs::remove_dir_all(&output);
}

// Unpack tests

#[test]
fn unpack_creates_valid_standalone_file() {
    let input = fixture_path("Andika Mini.glyphspackage");
    let output = std::env::temp_dir().join("unpack_valid_test.glyphs");
    let _ = fs::remove_file(&output);

    unpack::unpack(&input, &output).unwrap();

    let content = fs::read_to_string(&output).unwrap();
    plist::parse(plist::Root::Dict, &content).unwrap();

    let _ = fs::remove_file(&output);
}

#[test]
fn unpack_preserves_all_top_level_keys() {
    let original = fixture_path("Andika Mini.glyphs");
    let package = fixture_path("Andika Mini.glyphspackage");
    let output = std::env::temp_dir().join("unpack_keys_test.glyphs");
    let _ = fs::remove_file(&output);

    unpack::unpack(&package, &output).unwrap();

    let original_keys = dict_keys_from_file(&original);
    let unpacked_keys = dict_keys_from_file(&output);
    assert_eq!(original_keys, unpacked_keys);

    let _ = fs::remove_file(&output);
}

// Round-trip tests

#[test]
fn round_trip_pack_unpack_preserves_glyph_count() {
    let input = fixture_path("Andika Mini.glyphs");
    let package = std::env::temp_dir().join("rt_count_test.glyphspackage");
    let output = std::env::temp_dir().join("rt_count_test.glyphs");
    let _ = fs::remove_dir_all(&package);
    let _ = fs::remove_file(&output);

    let original_count = count_glyphs_in_file(&input);

    pack::pack(&input, &package, true).unwrap();
    unpack::unpack(&package, &output).unwrap();

    let roundtripped_count = count_glyphs_in_file(&output);
    assert_eq!(original_count, roundtripped_count);
    assert!(original_count > 0);

    let _ = fs::remove_dir_all(&package);
    let _ = fs::remove_file(&output);
}

#[test]
fn round_trip_pack_unpack_preserves_keys() {
    let input = fixture_path("Andika Mini.glyphs");
    let package = std::env::temp_dir().join("rt_keys_test.glyphspackage");
    let output = std::env::temp_dir().join("rt_keys_test.glyphs");
    let _ = fs::remove_dir_all(&package);
    let _ = fs::remove_file(&output);

    pack::pack(&input, &package, true).unwrap();
    unpack::unpack(&package, &output).unwrap();

    let original_keys = dict_keys_from_file(&input);
    let roundtripped_keys = dict_keys_from_file(&output);
    assert_eq!(original_keys, roundtripped_keys);

    let _ = fs::remove_dir_all(&package);
    let _ = fs::remove_file(&output);
}

#[test]
fn round_trip_unpack_pack_preserves_glyph_files() {
    let input = fixture_path("Andika Mini.glyphspackage");
    let standalone = std::env::temp_dir().join("rt_reverse_test.glyphs");
    let output = std::env::temp_dir().join("rt_reverse_test.glyphspackage");
    let _ = fs::remove_file(&standalone);
    let _ = fs::remove_dir_all(&output);

    unpack::unpack(&input, &standalone).unwrap();
    pack::pack(&standalone, &output, true).unwrap();

    let mut input_glyphs = list_glyph_filenames(&input.join("glyphs"));
    let mut output_glyphs = list_glyph_filenames(&output.join("glyphs"));
    input_glyphs.sort();
    output_glyphs.sort();
    assert_eq!(input_glyphs, output_glyphs);

    let _ = fs::remove_file(&standalone);
    let _ = fs::remove_dir_all(&output);
}

// Helpers

fn list_glyph_filenames(dir: &Path) -> Vec<String> {
    fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| {
            let e = e.unwrap();
            let path = e.path();
            if path.extension().is_some_and(|ext| ext == "glyph") {
                Some(e.file_name().to_string_lossy().into_owned())
            } else {
                None
            }
        })
        .collect()
}

fn dict_keys_from_file(path: &Path) -> Vec<String> {
    let content = fs::read_to_string(path).unwrap();
    let parsed = plist::parse(plist::Root::Dict, &content).unwrap();
    match parsed.value {
        plist::Value::Dict(pairs) => {
            let mut keys: Vec<String> = pairs.iter().map(|(k, _, _)| k.to_string()).collect();
            keys.sort();
            keys
        }
        _ => panic!("expected dict in {}", path.display()),
    }
}

fn count_glyphs_in_file(path: &Path) -> usize {
    let content = fs::read_to_string(path).unwrap();
    let parsed = plist::parse(plist::Root::Dict, &content).unwrap();
    match parsed.value {
        plist::Value::Dict(pairs) => {
            for (key, value, _) in &pairs {
                if key.as_ref() == "glyphs" {
                    if let plist::Value::Array(items) = &value.value {
                        return items.len();
                    }
                }
            }
            0
        }
        _ => 0,
    }
}
