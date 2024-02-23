use std::path::Path;
use crate::items::Program;

pub trait Language: Sync {
    fn lang_id(&self) -> &'static str;
    fn generate(&self, program: &Program, outdir: &Path);
}
