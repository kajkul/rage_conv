use std::{path::PathBuf, sync::OnceLock};

use clap::{builder::PossibleValue, ValueEnum};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Universal,
    Xml,
    Rage,
}

impl ValueEnum for Mode {
    fn value_variants<'a>() -> &'a [Self] {
        &[Mode::Universal, Mode::Xml, Mode::Rage]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Mode::Universal => PossibleValue::new("universal").help("Process both rage and xml"),
            Mode::Xml => PossibleValue::new("xml").help("Process only XML files"),
            Mode::Rage => PossibleValue::new("rage").help("Process only RAGE file formats"),
        })
    }
}

impl std::str::FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, false) {
                return Ok(*variant);
            }
        }
        Err(format!("invalid variant: {s}"))
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

#[derive(clap::Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value = ".")]
    pub input_dir: PathBuf,

    #[arg(short, long, default_value = "./out")]
    pub output_dir: PathBuf,

    #[arg(short, long, default_value_t = Mode::Universal)]
    pub mode: Mode,

    #[arg(short, long, default_value_t = false, help = "not implemented")]
    pub ytd_import_dir: bool,

    #[arg(short, long, default_value_t = false)]
    pub recursive: bool,
}

pub static ARGS: OnceLock<Args> = OnceLock::new();
