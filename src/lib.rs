pub mod pack;
pub mod plist;
pub mod unpack;

pub(crate) const FILE_EXT_GLYPH: &str = "glyph";

pub(crate) const FILE_PACKAGE_FONTINFO: &str = "fontinfo.plist";
pub(crate) const FILE_PACKAGE_ORDER: &str = "order.plist";
pub(crate) const FILE_PACKAGE_UI_STATE: &str = "UIState.plist";
pub(crate) const FILE_PACKAGE_GLYPHS: &str = "glyphs";

pub(crate) const KEY_DISPLAY_STRINGS_PACKAGE: &str = "displayStrings";
pub(crate) const KEY_DISPLAY_STRINGS_STANDALONE: &str = "DisplayStrings";
pub(crate) const KEY_GLYPH_NAME: &str = "glyphname";
pub(crate) const KEY_GLYPHS: &str = "glyphs";
