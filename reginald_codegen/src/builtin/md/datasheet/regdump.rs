use std::{
    collections::{BTreeMap, HashSet},
    fmt::Write,
    path::{Path, PathBuf},
};

use crate::{
    builtin::md::md_table,
    error::Error,
    regmap::{PhysicalRegister, RegisterMap, TypeAdr, TypeValue},
    utils::filename,
};

use super::generate_register_infos;

#[cfg(feature = "cli")]
use clap::Parser;
use serde::{Deserialize, Serialize};

// ====== Register Dump ========================================================

pub type RegDump = BTreeMap<TypeAdr, TypeValue>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged, deny_unknown_fields)]
enum RegDumpListingEntry {
    One(TypeValue),
    Multiple(Vec<TypeValue>),
}

fn read_regdump(path: &Path) -> Result<RegDump, Error> {
    let reader = std::fs::File::open(path)?;
    let regdump_listing: BTreeMap<TypeAdr, RegDumpListingEntry> = serde_yaml::from_reader(reader)?;

    let mut regdump = BTreeMap::new();
    for (start_adr, entry) in regdump_listing {
        match entry {
            RegDumpListingEntry::One(val) => {
                if regdump.insert(start_adr, val).is_some() {
                    return Err(Error::GeneratorError(format!(
                        "Regdump contains multiple values for address 0x{start_adr:X}."
                    )));
                }
            }
            RegDumpListingEntry::Multiple(vals) => {
                for (idx, val) in vals.iter().enumerate() {
                    let adr = start_adr + (idx as u64);
                    if regdump.insert(adr, *val).is_some() {
                        return Err(Error::GeneratorError(format!(
                            "Regdump contains multiple values for address 0x{adr:X}."
                        )));
                    }
                }
            }
        }
    }
    Ok(regdump)
}

// ====== Generator Options ====================================================

#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Parser))]
pub struct GeneratorOpts {
    /// Path to YAML register dump file
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub map: PathBuf,
}

// ====== Generator ============================================================

pub fn generate(out: &mut dyn Write, map: &RegisterMap, opts: &GeneratorOpts) -> Result<(), Error> {
    let regdump = read_regdump(&opts.map)?;

    let registers = map.physical_registers();
    let adrs = adrs_of_interest(&registers, &regdump);

    writeln!(out, "# {} Register Dump Decode Report", map.map_name)?;
    writeln!(out)?;
    writeln!(out, "## Register Map")?;
    generate_overview(out, map, &registers, &regdump, &adrs)?;

    writeln!(out)?;
    writeln!(out, "## Register Details")?;
    for adr in adrs {
        let (regs, val) = lookup_adr(&registers, &regdump, adr);
        for reg in regs {
            generate_register_infos(out, reg, val)?;
        }
    }

    Ok(())
}

fn generate_overview(
    out: &mut dyn Write,
    map: &RegisterMap,
    registers: &[PhysicalRegister],
    regdump: &RegDump,
    adrs: &Vec<TypeAdr>,
) -> Result<(), Error> {
    let mut rows = vec![];

    if let Some(input_file) = &map.from_file {
        writeln!(out)?;
        writeln!(out, "Generated from listing file: {}.", filename(input_file)?)?;
    }
    if let Some(author) = &map.author {
        writeln!(out)?;
        writeln!(out, "Listing file author: {author}")?;
    }
    if let Some(note) = &map.note {
        writeln!(out,)?;
        writeln!(out, "Listing file note:")?;
        writeln!(out, "```")?;
        for line in note.lines() {
            writeln!(out, "  {line}")?;
        }
        writeln!(out, "```")?;
    }

    rows.push(vec![
        "**Address**".to_string(),
        "**Register**".to_string(),
        "**Value**".to_string(),
        "**Brief**".to_string(),
    ]);
    for adr in adrs {
        let (regs, val) = lookup_adr(registers, regdump, *adr);
        let adr_str = format!("0x{adr:X}");
        let value_str = val.map(|x| format!("0x{x:X}")).unwrap_or_default();

        if regs.is_empty() {
            rows.push(vec![adr_str, "?".to_string(), value_str, "/".to_string()]);
        } else {
            for reg in regs {
                rows.push(vec![
                    adr_str.clone(),
                    reg.name.clone(),
                    value_str.clone(),
                    reg.template.docs.brief.clone().unwrap_or_default(),
                ]);
            }
        }
    }
    writeln!(out)?;
    md_table(out, &rows)?;
    Ok(())
}

fn adrs_of_interest(registers: &[PhysicalRegister], regdump: &RegDump) -> Vec<TypeAdr> {
    let mut adrs: HashSet<TypeAdr> = HashSet::new();

    for reg in registers {
        if let Some(adr) = reg.absolute_adr {
            adrs.insert(adr);
        }
    }

    for adr in regdump.keys() {
        adrs.insert(*adr);
    }

    let mut adrs: Vec<TypeAdr> = adrs.into_iter().collect();
    adrs.sort_unstable();
    adrs
}

fn lookup_adr<'a>(
    registers: &'a [PhysicalRegister<'a>],
    regdump: &RegDump,
    adr: TypeAdr,
) -> (Vec<&'a PhysicalRegister<'a>>, Option<TypeValue>) {
    let phyregs: Vec<&PhysicalRegister> = registers.iter().filter(|x| x.absolute_adr == Some(adr)).collect();
    let val = regdump.get(&adr);
    (phyregs, val.copied())
}
