use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::plist;
use crate::{
    FILE_EXT_GLYPH, FILE_PACKAGE_FONTINFO, FILE_PACKAGE_GLYPHS, FILE_PACKAGE_ORDER,
    FILE_PACKAGE_UI_STATE, KEY_DISPLAY_STRINGS_PACKAGE, KEY_DISPLAY_STRINGS_STANDALONE,
    KEY_GLYPH_NAME, KEY_GLYPHS,
};

fn read_glyph_files(glyphs_path: &Path) -> Result<(Vec<PathBuf>, HashMap<String, String>)> {
    let glyph_paths: Vec<PathBuf> = fs::read_dir(glyphs_path)
        .with_context(|| format!("cannot read glyph listing at {}", glyphs_path.display()))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("cannot read glyph entry at {}", glyphs_path.display()))?
        .into_iter()
        .filter(|entry| entry.path().extension() == Some(OsStr::new(FILE_EXT_GLYPH)))
        .map(|entry| entry.path())
        .collect();

    let glyphs: HashMap<String, String> = glyph_paths
        .par_iter()
        .map(|path| -> Result<(String, String)> {
            let glyph_code = fs::read_to_string(path)
                .with_context(|| format!("cannot read glyph at {}", path.display()))?;
            let glyphs_dict = plist::parse(plist::Root::Dict, &glyph_code)
                .with_context(|| format!("cannot parse glyph at {}", path.display()))?;
            let plist::Value::Dict(pairs) = glyphs_dict.value else {
                unreachable!()
            };

            for (key, slice, _) in pairs {
                if key.as_ref() == KEY_GLYPH_NAME {
                    let plist::Value::String(glyphname) = slice.value else {
                        bail!(
                            "non-string `{KEY_GLYPH_NAME}` value for glyph at {}",
                            path.display()
                        )
                    };
                    return Ok((glyphname.into_owned(), glyph_code));
                }
            }

            bail!("missing `{KEY_GLYPH_NAME}` in glyph at {}", path.display())
        })
        .collect::<Result<HashMap<_, _>>>()?;

    Ok((glyph_paths, glyphs))
}

pub fn unpack(in_path: &Path, out_path: &Path) -> Result<()> {
    // Read Font Info

    let fontinfo_path = in_path.join(FILE_PACKAGE_FONTINFO);
    let fontinfo_code = fs::read_to_string(&fontinfo_path)
        .with_context(|| format!("cannot read font info at {}", fontinfo_path.display()))?;
    let fontinfo = plist::parse(plist::Root::Dict, &fontinfo_code)
        .with_context(|| format!("cannot parse font info at {}", fontinfo_path.display()))?;
    let plist::Value::Dict(fontinfo) = fontinfo.value else {
        unreachable!()
    };
    let mut file_contents: Vec<(String, String)> = fontinfo
        .into_iter()
        .map(|(key, _, code)| (key.into_owned(), code.to_string()))
        .collect();

    // Read Order

    let order_path = in_path.join(FILE_PACKAGE_ORDER);
    let order_code = fs::read_to_string(&order_path)
        .with_context(|| format!("cannot read order at {}", order_path.display()))?;
    let order = plist::parse(plist::Root::Array, &order_code)
        .with_context(|| format!("cannot parse order at {}", order_path.display()))?;
    let order = match order.value {
        plist::Value::Array(x) => x
            .into_iter()
            .map(|entry| match entry.value {
                plist::Value::String(glyphname) => Ok(glyphname.into_owned()),
                _ => bail!("non-string glyph name in order at {}", order_path.display()),
            })
            .collect::<Result<Vec<String>>>()?,
        _ => unreachable!(),
    };

    // Read Glyphs

    let glyphs_path = in_path.join(FILE_PACKAGE_GLYPHS);
    let (_, glyphs) = read_glyph_files(&glyphs_path)?;
    let glyphs_code_value = order
        .iter()
        .map(|glyphname| match glyphs.get(glyphname.as_str()) {
            Some(glyph_code) => Ok(glyph_code.trim().to_string()),
            None => bail!(
                "missing glyph /{glyphname}; glyph appears in {} but not in {}",
                order_path.display(),
                glyphs_path.display()
            ),
        })
        .collect::<Result<Vec<String>>>()?
        .join(",");
    let glyphs_code = format!("{KEY_GLYPHS} = (\n{glyphs_code_value}\n);");
    file_contents.push((KEY_GLYPHS.to_string(), glyphs_code));

    // Read UI State

    let ui_state_path = in_path.join(FILE_PACKAGE_UI_STATE);
    if let Ok(ui_state_code) = fs::read_to_string(&ui_state_path) {
        let ui_state = plist::parse(plist::Root::Dict, &ui_state_code)
            .with_context(|| format!("cannot parse UI state at {}", ui_state_path.display()))?;
        let plist::Value::Dict(ui_state) = ui_state.value else {
            unreachable!()
        };

        for (key, slice, code) in ui_state {
            let (key, code) = match key.as_ref() {
                KEY_DISPLAY_STRINGS_PACKAGE => (
                    KEY_DISPLAY_STRINGS_STANDALONE.to_string(),
                    format!("{KEY_DISPLAY_STRINGS_STANDALONE} = (\n{}\n);", slice.code),
                ),
                _ => (key.into_owned(), code.to_string()),
            };

            file_contents.push((key, code));
        }
    }

    // Write Standalone Glyphs File

    file_contents.sort_by(|(a, _), (b, _)| a.cmp(b));
    plist::write_dict_file(
        out_path,
        &file_contents
            .iter()
            .map(|(_, x)| x.as_str())
            .collect::<Vec<_>>(),
    )?;

    Ok(())
}
