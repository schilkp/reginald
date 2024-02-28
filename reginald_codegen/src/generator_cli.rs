use std::path::Path;

use crate::{error::GeneratorError, regmap::RegisterMap};

pub trait GeneratorCLI {
    fn generate(
        &self,
        map: RegisterMap,
        output_file_name: &Path,
        args: Vec<String>,
    ) -> Result<Vec<String>, GeneratorError>;

    fn help(&self, args: Vec<String>);

    fn description(&self) -> String;
}
