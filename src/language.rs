use std::path::{Path, PathBuf};
use crate::items::Program;

pub trait Language: Sync {
    fn lang_id(&self) -> &'static str;
    fn extension(&self) -> &'static str;
    fn generate(&self, program: &Program, outfile: &Path);
}
