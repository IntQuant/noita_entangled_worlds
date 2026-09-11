use noita_api::raw::mod_text_file_get_content;
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::collections::hash_map::Entry;

/// Animation names read out of sprite xml files, so a sprite's animation can
/// go over the wire as a position in its file's list rather than as a string.
///
/// The list is every `name="..."` value in the file, in order - which also
/// picks up things like the `filename="..."` attribute. Only the positions have
/// to agree, and both peers parse the same file the same way, so they do.
#[derive(Default)]
pub(crate) struct SpriteAnimations {
    files: FxHashMap<Cow<'static, str>, Vec<String>>,
}

impl SpriteAnimations {
    /// Where `animation` sits in `file`'s list, or `u16::MAX` if it isn't there.
    pub(crate) fn index_of(
        &mut self,
        file: Cow<'static, str>,
        animation: &str,
    ) -> eyre::Result<u16> {
        Ok(self
            .names(file)?
            .iter()
            .position(|name| name == animation)
            .and_then(|i| u16::try_from(i).ok())
            .unwrap_or(u16::MAX))
    }

    /// The name at `index` in `file`'s list, if the list is that long.
    pub(crate) fn name_at(
        &mut self,
        file: Cow<'static, str>,
        index: u16,
    ) -> eyre::Result<Option<&str>> {
        Ok(self.names(file)?.get(index as usize).map(String::as_str))
    }

    fn names(&mut self, file: Cow<'static, str>) -> eyre::Result<&[String]> {
        match self.files.entry(file) {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(entry) => {
                let content = mod_text_file_get_content(entry.key().clone())?;
                // A value missing its closing quote means a malformed file.
                // Stop the list there instead of panicking; every peer reading
                // the same file stops at the same place.
                let names = content
                    .split("name=\"")
                    .skip(1)
                    .map_while(|piece| piece.split_once('"').map(|(name, _)| name.to_string()))
                    .collect();
                Ok(entry.insert(names))
            }
        }
    }
}
