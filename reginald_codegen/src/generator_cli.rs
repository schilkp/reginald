use std::path::PathBuf;

use crate::{error::Error, regmap::RegisterMap};

pub trait GeneratorCLI {
    fn generate(
        &self,
        map: RegisterMap,
        output_file_name: PathBuf,
        args: Vec<String>,
    ) -> Result<Vec<String>, Error>;

    fn help(&self, args: Vec<String>);

    fn description(&self) -> String;
}
