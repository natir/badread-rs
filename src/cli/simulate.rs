//! All stuff relate to simulate subcommand

use anyhow::Context;

/// Store quantity as coverage of number of base
#[derive(Debug, PartialEq, Eq)]
pub struct Quantity {
    coverage: u64,
    base: Option<u64>,
}

impl std::str::FromStr for Quantity {
    type Err = crate::error::Cli;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.as_bytes() {
            &[rest @ .., b'x'] => match std::str::from_utf8(rest) {
                Ok(number) => match u64::from_str(number) {
                    Ok(n) => Ok(Quantity {
                        coverage: n,
                        base: None,
                    }),
                    Err(_) => Err(crate::error::Cli::CantParseQuantity),
                },
                Err(_) => Err(crate::error::Cli::CantParseQuantity),
            },
            &[rest @ .., b'G'] => match std::str::from_utf8(rest) {
                Ok(number) => match u64::from_str(number) {
                    Ok(n) => Ok(Quantity {
                        coverage: 0,
                        base: Some(n * 1_000_000_000),
                    }),
                    Err(_) => Err(crate::error::Cli::CantParseQuantity),
                },
                Err(_) => Err(crate::error::Cli::CantParseQuantity),
            },
            &[rest @ .., b'M'] => match std::str::from_utf8(rest) {
                Ok(number) => match u64::from_str(number) {
                    Ok(n) => Ok(Quantity {
                        coverage: 0,
                        base: Some(n * 1_000_000),
                    }),
                    Err(_) => Err(crate::error::Cli::CantParseQuantity),
                },
                Err(_) => Err(crate::error::Cli::CantParseQuantity),
            },
            &[rest @ .., b'K'] => match std::str::from_utf8(rest) {
                Ok(number) => match u64::from_str(number) {
                    Ok(n) => Ok(Quantity {
                        coverage: 0,
                        base: Some(n * 1_000),
                    }),
                    Err(_) => Err(crate::error::Cli::CantParseQuantity),
                },
                Err(_) => Err(crate::error::Cli::CantParseQuantity),
            },
            rest => match std::str::from_utf8(rest) {
                Ok(number) => match u64::from_str(number) {
                    Ok(n) => Ok(Quantity {
                        coverage: 0,
                        base: Some(n),
                    }),
                    Err(_) => Err(crate::error::Cli::CantParseQuantity),
                },
                Err(_) => Err(crate::error::Cli::CantParseQuantity),
            },
        }
    }
}

impl Quantity {
    /// Convert Quantity in a number of base, if base is set return just number of base else return genome_length times coverage
    pub fn number_of_base(&self, genome_length: u64) -> u64 {
        match self.base {
            Some(n) => n,
            None => genome_length * self.coverage,
        }
    }
}

/// Store a pair of value, can be parse from str if it's match with \d+,\d+
#[derive(Debug, PartialEq, Eq)]
pub struct Duo(pub u64, pub u64);

impl std::str::FromStr for Duo {
    type Err = crate::error::Cli;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements: Vec<&str> = s.split(',').collect();

        if elements.len() != 2 {
            Err(crate::error::Cli::CantParseDuo)
        } else {
            match (u64::from_str(elements[0]), u64::from_str(elements[1])) {
                (Ok(a), Ok(b)) => Ok(Duo(a, b)),
                _ => Err(crate::error::Cli::CantParseDuo),
            }
        }
    }
}

/// Store a trio of value, can be parse from str if it's match with \d+,\d+,\d++
#[derive(Debug, PartialEq, Eq)]
pub struct Trio(pub u64, pub u64, pub u64);

impl std::str::FromStr for Trio {
    type Err = crate::error::Cli;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements: Vec<&str> = s.split(',').collect();

        if elements.len() != 3 {
            Err(crate::error::Cli::CantParseTrio)
        } else {
            match (
                u64::from_str(elements[0]),
                u64::from_str(elements[1]),
                u64::from_str(elements[2]),
            ) {
                (Ok(a), Ok(b), Ok(c)) => Ok(Trio(a, b, c)),
                _ => Err(crate::error::Cli::CantParseTrio),
            }
        }
    }
}

/// Found path to model file
///
/// If value is path to a file just return value else search in `python -c "import sys; print(','.join(sys.path))"`
pub fn found_model(value: String, model_type: String) -> anyhow::Result<std::path::PathBuf> {
    let path = std::path::PathBuf::from(&value);

    if path.is_file() {
        log::info!("Model parameter is a file");
        Ok(path)
    } else {
        let result = std::process::Command::new("python")
            .args(&["-c", "import sys; print(','.join(sys.path))"])
            .output()
            .with_context(|| crate::error::Cli::CantFoundModelPath)?;

        let paths = std::str::from_utf8(&result.stdout)
            .with_context(|| crate::error::Cli::CantFoundModelPath)?;

        for path in paths.split(',') {
            let mut local_path = std::path::PathBuf::from(path);
            local_path.push(format!("badread/{}_models/", model_type));
            local_path.push(&value);
            local_path.set_extension("gz");

            if local_path.is_file() {
                log::info!("Model found in python sys.path");
                return Ok(local_path);
            }
        }

        Err(anyhow::anyhow!(crate::error::Cli::CantFoundModelPath))
    }
}

/// Struct use to parse simulate subcommand argument
#[derive(clap::Parser, Debug)]
#[clap(about = "Generate fake long read")]
pub struct Command {
    /// Path to reference fasta (can be gzipped, bzip2ped, xzped)
    #[clap(long = "reference", required = true)]
    pub reference_path: String,

    /// Path where read is write
    #[clap(long = "output")]
    pub output_path: Option<String>,

    /// Either an absolute value (e.g. 250M) or a relative depth (e.g. 25x)
    #[clap(long = "quantity", required = true)]
    pub quantity: Quantity,

    /// Fragment length distribution (mean and stdev)
    #[clap(long = "length", default_value = "15000,13000")]
    pub length: Duo,

    /// Sequencing identity distribution (mean, max and stdev)
    #[clap(long = "identity", default_value = "85,95,5")]
    pub identity: Trio,

    /// Path to an error model file
    #[clap(long = "error_model", default_value = "nanopore2020")]
    pub error_model: String,

    /// Path to an quality score model file
    #[clap(long = "qscore_model", default_value = "nanopore2020")]
    pub qscore_model: String,

    /// Random number generator seed for deterministic output (default: different output each time)
    #[clap(long = "seed")]
    pub seed: Option<u64>,

    /// Adapter parameters for read starts (rate and amount)
    #[clap(long = "start_adapter", default_value = "90,60")]
    pub start_adapter: Duo,

    /// Adapter parameters for read ends (rate and amount)
    #[clap(long = "end_adapter", default_value = "50,20")]
    pub end_adapter: Duo,

    /// Adapter parameters for read starts
    #[clap(
        long = "start_adapter_seq",
        default_value = "AATGTACTTCGTTCAGTTACGTATTGCT"
    )]
    pub start_adapter_seq: String,

    /// Adapter parameters for read ends
    #[clap(long = "end_adapter_seq", default_value = "GCAATACGTAACTGAACGAAGT")]
    pub end_adapter_seq: String,

    /// This percentage of reads wil be low complexity junk
    #[clap(long = "junk_reads", default_value = "1")]
    pub junk: f64,

    /// This percentage of reads wil be random sequence
    #[clap(long = "random_reads", default_value = "1")]
    pub random: f64,

    /// Percentage at which separate fragments join together
    #[clap(long = "chimera", default_value = "1")]
    pub chimera: f64,

    /// Read glitch parameters (rate, size and skip)
    #[clap(long = "glitches", default_value = "10000,25,25")]
    pub glitches: Trio,

    /// If set, then small circular plasmids are lost when the fragment length is too high (default: small plasmids are included regardless of fragment length)
    #[clap(long = "small_plasmid_bias")]
    pub small_plasmid_bias: bool,

    /// Number of base, rustyread can store in ram before write in output in absolute value (e.g. 250M) or a relative depth (e.g. 25x)
    #[clap(long = "number_base_store")]
    pub nb_base_store: Option<Quantity>,
}

#[cfg(test)]
mod t {
    use super::*;

    use std::str::FromStr;

    #[test]
    fn parse_quantity() {
        assert_eq!(
            Quantity::from_str("50").unwrap(),
            Quantity {
                coverage: 0,
                base: Some(50)
            }
        );
        assert_eq!(
            Quantity::from_str("50K").unwrap(),
            Quantity {
                coverage: 0,
                base: Some(50_000)
            }
        );
        assert_eq!(
            Quantity::from_str("50M").unwrap(),
            Quantity {
                coverage: 0,
                base: Some(50_000_000)
            }
        );
        assert_eq!(
            Quantity::from_str("50G").unwrap(),
            Quantity {
                coverage: 0,
                base: Some(50_000_000_000)
            }
        );
        assert_eq!(
            Quantity::from_str("50x").unwrap(),
            Quantity {
                coverage: 50,
                base: None
            }
        );

        assert!(Quantity::from_str("").is_err());
        assert!(Quantity::from_str("50z").is_err());
        assert!(Quantity::from_str("bépo50").is_err());
        assert!(Quantity::from_str("bépo50K").is_err());
        assert!(Quantity::from_str("bépo50M").is_err());
        assert!(Quantity::from_str("bépo50G").is_err());
        assert!(Quantity::from_str("bépo50x").is_err());
    }

    #[test]
    fn quantity2number_of_base() {
        assert_eq!(Quantity::from_str("50").unwrap().number_of_base(50), 50);
        assert_eq!(
            Quantity::from_str("50K").unwrap().number_of_base(50),
            50_000
        );
        assert_eq!(
            Quantity::from_str("50M").unwrap().number_of_base(50),
            50_000_000
        );
        assert_eq!(
            Quantity::from_str("50G").unwrap().number_of_base(50),
            50_000_000_000
        );
        assert_eq!(Quantity::from_str("50x").unwrap().number_of_base(50), 2500);
    }

    #[test]
    fn parse_pair() {
        assert_eq!(Duo::from_str("50,45").unwrap(), Duo(50, 45));

        assert!(Duo::from_str("50,43,").is_err());
        assert!(Duo::from_str("50,43,74").is_err());
        assert!(Duo::from_str("50,43,,").is_err());
        assert!(Duo::from_str("50,,43").is_err());
        assert!(Duo::from_str(",,").is_err());

        assert!(Duo::from_str("bépo50,43").is_err());
        assert!(Duo::from_str("50,bépo43").is_err());
    }

    #[test]
    fn parse_trio() {
        assert_eq!(Trio::from_str("50,45,74").unwrap(), Trio(50, 45, 74));

        assert!(Trio::from_str("50,43").is_err());
        assert!(Trio::from_str("50,43,,").is_err());
        assert!(Trio::from_str("50,,43").is_err());
        assert!(Trio::from_str(",,,").is_err());

        assert!(Trio::from_str("bépo50,43,74").is_err());
        assert!(Trio::from_str("50,bépo43,74").is_err());
        assert!(Trio::from_str("50,43,bépo74").is_err());
    }
}
