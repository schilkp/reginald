use clap::Parser;
use reginald_codegen::{
    builtin::c::{self, funcpack::Element},
    regmap::TypeBitwidth,
    utils::Endianess,
};

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    /// Generate functions and enums with the given endianess.
    ///
    /// May be given multiple times. If not specified, both endianess
    /// versions will be generated.
    #[arg(long)]
    #[arg(action = clap::ArgAction::Append)]
    #[arg(verbatim_doc_comment)]
    #[arg(conflicts_with("dont_generate"))]
    pub endian: Vec<Endianess>,

    /// For other endianess, generate only simple functions that defers to this implementation.
    ///
    /// If generating both endianess versions, only generate one complete
    /// function implementation and have the other endianess defer to this
    #[arg(long)]
    #[arg(action = clap::ArgAction::Set)]
    #[arg(verbatim_doc_comment)]
    pub defer_to_endian: Option<Endianess>,

    /// Make register structs bitfields to reduce their memory size
    ///
    /// May reduce performance. Note that their memory layout will not match the actual register
    /// and the (un)packing functions must still be used.
    #[arg(long)]
    #[arg(action = clap::ArgAction::Set)]
    #[arg(default_value_t = Self::default().registers_as_bitfields)]
    #[arg(verbatim_doc_comment)]
    pub registers_as_bitfields: bool,

    /// Max enum bitwidth before it is represented using macros instead of an enum.
    ///
    /// Set to zero to have all enums be represented using macros.
    #[arg(long)]
    #[arg(action = clap::ArgAction::Set)]
    #[arg(default_value_t = Self::default().max_enum_bitwidth)]
    #[arg(verbatim_doc_comment)]
    pub max_enum_bitwidth: TypeBitwidth,

    /// Header file that should be included at the top of the generated header
    ///
    /// May be given multiple times.
    #[arg(long)]
    #[arg(action = clap::ArgAction::Append)]
    #[arg(verbatim_doc_comment)]
    pub add_include: Vec<String>,

    /// Make all functions static inline.
    ///
    /// May be disabled if splitting code into header and source.
    #[arg(long)]
    #[arg(action = clap::ArgAction::Set)]
    #[arg(default_value_t = Self::default().funcs_static_inline)]
    #[arg(verbatim_doc_comment)]
    pub funcs_static_inline: bool,

    /// Generate function prototypes instead of full implementations.
    ///
    /// May be enabled if splitting code into header and source.
    #[arg(long)]
    #[arg(action = clap::ArgAction::Set)]
    #[arg(default_value_t = Self::default().funcs_as_prototypes)]
    #[arg(verbatim_doc_comment)]
    pub funcs_as_prototypes: bool,

    /// Surround file with a clang-format off guard
    #[arg(long)]
    #[arg(action = clap::ArgAction::Set)]
    #[arg(default_value_t = Self::default().clang_format_guard)]
    #[arg(verbatim_doc_comment)]
    pub clang_format_guard: bool,

    /// Generate include guard
    #[arg(long)]
    #[arg(action = clap::ArgAction::Set)]
    #[arg(default_value_t = Self::default().include_guards)]
    #[arg(verbatim_doc_comment)]
    pub include_guards: bool,

    /// Only generate a subset of the elements/sections usually included in
    /// a complete output file.
    ///
    /// This option is mutually exclusive with 'dont_generate'
    /// If this option is not given, all elements are generated. This option
    /// may be given multiple times.
    /// Note that different components depend on each other. It is up to the
    /// user to generate all required sections, or add includes that provide
    /// those elements.
    #[arg(long)]
    #[arg(action = clap::ArgAction::Append)]
    #[arg(verbatim_doc_comment)]
    #[arg(conflicts_with("dont_generate"))]
    pub only_generate: Vec<Element>,

    /// Skip generation of some element/section usually included in a complete
    /// output file.
    ///
    /// This option is mutually exclusive with 'only_generate'
    /// Note that different components depend on each other. It is up to the
    /// user to generate all required sections, or add includes that provide
    /// those elements.
    #[arg(long)]
    #[arg(action = clap::ArgAction::Append)]
    #[arg(verbatim_doc_comment)]
    #[arg(conflicts_with("only_generate"))]
    pub dont_generate: Vec<Element>,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            endian: vec![],
            defer_to_endian: None,
            registers_as_bitfields: false,
            max_enum_bitwidth: 31,
            add_include: vec![],
            funcs_static_inline: true,
            funcs_as_prototypes: false,
            clang_format_guard: true,
            include_guards: true,
            only_generate: vec![],
            dont_generate: vec![],
        }
    }
}

impl From<&Cli> for c::funcpack::GeneratorOpts {
    fn from(cli: &Cli) -> Self {
        let mut to_generate = c::funcpack::GeneratorOpts::default().to_generate;

        if !cli.only_generate.is_empty() {
            for to_gen in &cli.only_generate {
                to_generate.insert(*to_gen);
            }
        } else if !cli.dont_generate.is_empty() {
            for to_skip in &cli.only_generate {
                to_generate.remove(to_skip);
            }
        };

        c::funcpack::GeneratorOpts {
            endian: cli.endian.clone(),
            defer_to_endian: cli.defer_to_endian,
            registers_as_bitfields: cli.registers_as_bitfields,
            max_enum_bitwidth: cli.max_enum_bitwidth,
            add_include: cli.add_include.clone(),
            funcs_static_inline: cli.funcs_static_inline,
            funcs_as_prototypes: cli.funcs_as_prototypes,
            clang_format_guard: cli.clang_format_guard,
            include_guards: cli.include_guards,
            to_generate,
        }
    }
}
