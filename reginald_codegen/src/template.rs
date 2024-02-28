use std::{collections::HashMap, fmt::Write, path::Path};

use tera::Tera;

use crate::{
    error::GeneratorError,
    regmap::RegisterMap,
    utils::{self, filename},
};

pub fn render_template(
    out: &dyn Write,
    template: &str,
    map: &RegisterMap,
    output_file: &Path,
    args: Vec<String>,
) -> Result<(), GeneratorError> {
    let mut tera = Tera::default();
    tera.add_raw_template("generator", template)?;

    tera.register_function(
        "c_sanitize",
        TeraFunc {
            is_safe: true,
            func: |x| {
                let inp = match x.get("") {
                    Some(tera::Value::String(s)) => s,
                    _ => Err("Argument error: c_sanitize expects a string.".to_string())?,
                };
                Ok(utils::c_sanitize(inp).into())
            },
        },
    );

    tera.register_function(
        "c_fitting_unsigned_type",
        TeraFunc {
            is_safe: true,
            func: |x| {
                let inp = match x.get("") {
                    Some(tera::Value::Number(tera::Number)) => s,
                    _ => Err("Argument error: c_sanitize expects a number.".to_string())?,
                };
                match utils::c_fitting_unsigned_type(inp.into()) {
                    Ok(val) => Ok(val.into()),
                    Err(_) => todo!(),
                }
            }
        },
    );


    let mut context = tera::Context::new();
    context.insert("args", &args);
    context.insert("map", &map);
    context.insert("output_file_full", &output_file.to_string_lossy().to_string());
    context.insert("output_file", &filename(&output_file)?);

    // c_sanitize=reginald.utils.c_sanitize,
    // c_fitting_unsigned_type=reginald.utils.c_fitting_unsigned_type,
    // str_pad_to_length=reginald.utils.str_pad_to_length,
    // hex=hex,

    // // Render the template with the given context
    // let rendered = tera.render("hello", &context).unwrap();

    Ok(())
}

struct TeraFunc {
    is_safe: bool,
    func: fn(args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value>,
}

impl tera::Function for TeraFunc {
    fn call(&self, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        (self.func)(args)
    }

    fn is_safe(&self) -> bool {
        self.is_safe
    }
}
