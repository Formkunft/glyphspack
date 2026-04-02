use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use regex::{Captures, Regex};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use crate::{
    FILE_EXT_GLYPH, FILE_PACKAGE_FONTINFO, FILE_PACKAGE_GLYPHS, FILE_PACKAGE_ORDER,
    FILE_PACKAGE_UI_STATE, KEY_DISPLAY_STRINGS_PACKAGE, KEY_DISPLAY_STRINGS_STANDALONE,
    KEY_GLYPH_NAME, KEY_GLYPHS, plist,
};

static INIT_DOT_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\.").unwrap());
static CAPITAL_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[A-Z]").unwrap());

const RESERVED_NAMES: &[&str] = &[
    "CON", "CLOCK$", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
    "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8",
];

fn is_reserved(name: &str) -> bool {
    RESERVED_NAMES.iter().any(|r| r.eq_ignore_ascii_case(name))
}

pub fn glyph_filename(glyphname: &str) -> String {
    let file_stem = INIT_DOT_REGEX.replacen(glyphname, 1, "_");
    let file_stem = CAPITAL_REGEX.replace_all(file_stem.as_ref(), |captures: &Captures| {
        format!(
            "{}_",
            captures.get(0).unwrap().as_str().to_ascii_uppercase()
        )
    });

    let file_stem: String = file_stem
        .split('.')
        .map(|part| {
            if is_reserved(part) {
                format!("_{part}")
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(".");

    format!("{file_stem}.{FILE_EXT_GLYPH}")
}

pub fn pack(in_path: &Path, out_path: &Path, force: bool) -> Result<()> {
    // Read Standalone File

    let standalone_code = fs::read_to_string(in_path)
        .with_context(|| format!("cannot read font info at {}", in_path.display()))?;
    let standalone = plist::parse(plist::Root::Dict, &standalone_code)
        .with_context(|| format!("cannot parse font info at {}", in_path.display()))?;
    let plist::Value::Dict(standalone) = standalone.value else {
        unreachable!()
    };

    let mut fontinfo: Vec<&str> = Vec::new();
    let mut order: Vec<String> = Vec::new();
    let mut ui_state: Vec<String> = Vec::new();
    let mut glyphs: Vec<(String, &str)> = Vec::new();

    for (key, slice, code) in standalone {
        match key.as_ref() {
            KEY_DISPLAY_STRINGS_STANDALONE => {
                let code = format!("{KEY_DISPLAY_STRINGS_PACKAGE} = (\n{}\n);", slice.code);
                ui_state.push(code);
            }
            KEY_GLYPHS => {
                let plist::Value::Array(glyph_slices) = slice.value else {
                    bail!("non-array `{KEY_GLYPHS}` in {}", in_path.display())
                };
                for glyph_slice in glyph_slices {
                    let plist::Value::Dict(glyph) = glyph_slice.value else {
                        bail!("non-dict glyph in {}", in_path.display())
                    };
                    let mut glyph_name: Option<String> = None;
                    for (key, slice, _) in glyph {
                        if key.as_ref() == KEY_GLYPH_NAME {
                            match slice.value {
                                plist::Value::String(name) => glyph_name = Some(name.into_owned()),
                                _ => {
                                    bail!("non-string `{KEY_GLYPH_NAME}` in {}", in_path.display())
                                }
                            }
                        }
                    }
                    let glyph_name = glyph_name.with_context(|| {
                        format!(
                            "missing `{KEY_GLYPH_NAME}` in glyph in {}",
                            in_path.display()
                        )
                    })?;

                    order.push(glyph_name.clone());
                    glyphs.push((glyph_name, glyph_slice.code));
                }
            }
            _ => {
                fontinfo.push(code);
            }
        }
    }

    // Create Directories

    if force && out_path.is_dir() {
        fs::remove_dir_all(out_path).with_context(|| {
            format!(
                "cannot overwrite existing directory at {}",
                out_path.display()
            )
        })?;
    }

    fs::create_dir_all(out_path)
        .with_context(|| format!("cannot create package at {}", out_path.display()))?;

    let glyphs_path = out_path.join(FILE_PACKAGE_GLYPHS);
    fs::create_dir(&glyphs_path).with_context(|| {
        format!(
            "cannot create glyphs directory at {}",
            glyphs_path.display()
        )
    })?;

    // Write Font Info

    let fontinfo_path = out_path.join(FILE_PACKAGE_FONTINFO);
    plist::write_dict_file(&fontinfo_path, &fontinfo)?;

    // Write Order

    let order_path = out_path.join(FILE_PACKAGE_ORDER);
    let order_refs: Vec<&str> = order.iter().map(String::as_str).collect();
    plist::write_array_file(&order_path, &order_refs)?;

    // Write UI State

    if !ui_state.is_empty() {
        let ui_state_path = out_path.join(FILE_PACKAGE_UI_STATE);
        plist::write_dict_file(
            &ui_state_path,
            &ui_state.iter().map(|x| &**x).collect::<Vec<_>>(),
        )?;
    }

    // Write Glyphs

    glyphs.into_par_iter().try_for_each(|(glyphname, code)| {
        let glyph_path = glyphs_path.join(glyph_filename(&glyphname));
        plist::write_dict_file(&glyph_path, &[code])?;
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename_lowercase() {
        assert_eq!(glyph_filename("a"), "a.glyph");
    }

    #[test]
    fn filename_uppercase() {
        assert_eq!(glyph_filename("A"), "A_.glyph");
    }

    #[test]
    fn filename_uppercase_pair() {
        assert_eq!(glyph_filename("AE"), "A_E_.glyph");
    }

    #[test]
    fn filename_uppercase_then_lower() {
        assert_eq!(glyph_filename("Ae"), "A_e.glyph");
    }

    #[test]
    fn filename_all_lower() {
        assert_eq!(glyph_filename("ae"), "ae.glyph");
    }

    #[test]
    fn filename_lower_then_upper() {
        assert_eq!(glyph_filename("aE"), "aE_.glyph");
    }

    #[test]
    fn filename_dotted_suffix_lower() {
        assert_eq!(glyph_filename("a.alt"), "a.alt.glyph");
    }

    #[test]
    fn filename_dotted_suffix_upper_base() {
        assert_eq!(glyph_filename("A.alt"), "A_.alt.glyph");
    }

    #[test]
    fn filename_dotted_suffix_upper_in_suffix() {
        assert_eq!(glyph_filename("A.Alt"), "A_.A_lt.glyph");
    }

    #[test]
    fn filename_dotted_suffix_mixed() {
        assert_eq!(glyph_filename("A.aLt"), "A_.aL_t.glyph");
    }

    #[test]
    fn filename_dotted_suffix_trailing_upper() {
        assert_eq!(glyph_filename("A.alT"), "A_.alT_.glyph");
    }

    #[test]
    fn filename_underscore_both_upper() {
        assert_eq!(glyph_filename("T_H"), "T__H_.glyph");
    }

    #[test]
    fn filename_underscore_upper_lower() {
        assert_eq!(glyph_filename("T_h"), "T__h.glyph");
    }

    #[test]
    fn filename_underscore_both_lower() {
        assert_eq!(glyph_filename("t_h"), "t_h.glyph");
    }

    #[test]
    fn filename_ligature_all_upper() {
        assert_eq!(glyph_filename("F_F_I"), "F__F__I_.glyph");
    }

    #[test]
    fn filename_ligature_all_lower() {
        assert_eq!(glyph_filename("f_f_i"), "f_f_i.glyph");
    }

    #[test]
    fn filename_complex_ligature_with_suffix() {
        assert_eq!(glyph_filename("Aacute_V.swash"), "A_acute_V_.swash.glyph");
    }

    #[test]
    fn filename_initial_dot() {
        assert_eq!(glyph_filename(".notdef"), "_notdef.glyph");
    }

    // Reserved Windows filenames

    #[test]
    fn filename_reserved_con() {
        assert_eq!(glyph_filename("con"), "_con.glyph");
    }

    #[test]
    fn filename_reserved_con_uppercase() {
        assert_eq!(glyph_filename("CON"), "C_O_N_.glyph");
    }

    #[test]
    fn filename_reserved_con_dotted() {
        assert_eq!(glyph_filename("con.alt"), "_con.alt.glyph");
    }

    #[test]
    fn filename_reserved_in_suffix() {
        assert_eq!(glyph_filename("alt.con"), "alt._con.glyph");
    }

    #[test]
    fn filename_not_reserved() {
        assert_eq!(glyph_filename("convent"), "convent.glyph");
    }
}
