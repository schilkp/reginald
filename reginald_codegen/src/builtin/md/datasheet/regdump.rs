use std::{
    collections::{BTreeMap, HashSet},
    fmt::Write,
};

use crate::{
    builtin::md::md_table,
    error::GeneratorError,
    regmap::{PhysicalRegister, RegisterMap, TypeAdr, TypeValue},
};

use super::generate_register_infos;

pub type RegDump = BTreeMap<TypeAdr, TypeValue>;

pub fn generate(out: &mut dyn Write, map: &RegisterMap, regdump: &RegDump) -> Result<(), GeneratorError> {
    let registers = map.physical_registers();
    let adrs = adrs_of_interest(&registers, &regdump);

    writeln!(out, "# {} Register Dump Decode Report", map.map_name)?;
    writeln!(out, "")?;
    writeln!(out, "## Register Map")?;
    writeln!(out, "")?;
    generate_overview(out, &registers, regdump, &adrs)?;

    writeln!(out, "")?;
    writeln!(out, "## Register Details")?;
    for adr in adrs {
        let (regs, val) = lookup_adr(&registers, regdump, adr);
        for reg in regs {
            generate_register_infos(out, reg, val)?;
        }
    }

    Ok(())
}

fn generate_overview(
    out: &mut dyn Write,
    registers: &Vec<PhysicalRegister>,
    regdump: &RegDump,
    adrs: &Vec<TypeAdr>,
) -> Result<(), GeneratorError> {
    let mut rows = vec![];

    rows.push(vec![
        "*Address*".to_string(),
        "*Register*".to_string(),
        "*Value*".to_string(),
        "*Brief*".to_string(),
    ]);
    for adr in adrs {
        let (regs, val) = lookup_adr(registers, regdump, *adr);
        let adr_str = format!("0x{:X}", adr);
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
    md_table(out, &rows)?;
    Ok(())
}

fn adrs_of_interest(registers: &Vec<PhysicalRegister>, regdump: &RegDump) -> Vec<TypeAdr> {
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
    adrs.sort();
    adrs
}

fn lookup_adr<'a>(
    registers: &'a Vec<PhysicalRegister<'a>>,
    regdump: &RegDump,
    adr: TypeAdr,
) -> (Vec<&'a PhysicalRegister<'a>>, Option<TypeValue>) {
    let phyregs: Vec<&PhysicalRegister> = registers.iter().filter(|x| x.absolute_adr == Some(adr)).collect();
    let val = regdump.get(&adr);
    (phyregs, val.copied())
}
