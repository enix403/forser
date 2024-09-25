use crate::items::Program;
use std::path::{Path, PathBuf};

pub trait Language: Sync {
    fn lang_id(&self) -> &'static str;
    fn extension(&self) -> &'static str;
    fn generate(&self, program: &Program, outfile: &Path);
}
