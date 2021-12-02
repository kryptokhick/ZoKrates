use crate::constants::{FLATTENED_CODE_DEFAULT_PATH, MPC_DEFAULT_PATH};
use clap::{App, Arg, ArgMatches, SubCommand};
use phase2::MPCParameters;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use zokrates_core::ir;
use zokrates_core::ir::ProgEnum;
use zokrates_core::proof_system::bellman::Computation;
use zokrates_field::Bn128Field;

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name("verify")
        .about("Verifies correctness of MPC parameters, given a circuit instance")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .help("Path of the MPC parameters")
                .value_name("FILE")
                .takes_value(true)
                .required(false)
                .default_value(MPC_DEFAULT_PATH),
        )
        .arg(
            Arg::with_name("circuit")
                .short("c")
                .long("circuit")
                .help("Path of the circuit binary")
                .value_name("FILE")
                .takes_value(true)
                .required(false)
                .default_value(FLATTENED_CODE_DEFAULT_PATH),
        )
        .arg(
            Arg::with_name("radix-path")
                .short("r")
                .long("radix-dir")
                .help("Path to the radix file containing parameters for a circuit depth of 2^n (phase1radix2m{n})")
                .value_name("PATH")
                .takes_value(true)
                .required(true),
        )
}

pub fn exec(sub_matches: &ArgMatches) -> Result<(), String> {
    // read compiled program
    let path = Path::new(sub_matches.value_of("circuit").unwrap());
    let file =
        File::open(&path).map_err(|why| format!("Could not open `{}`: {}", path.display(), why))?;

    let mut reader = BufReader::new(file);

    match ProgEnum::deserialize(&mut reader)? {
        ProgEnum::Bn128Program(p) => cli_mpc_verify(p, sub_matches),
        _ => Err("Current protocol only supports bn128 programs".into()),
    }
}

fn cli_mpc_verify(ir_prog: ir::Prog<Bn128Field>, sub_matches: &ArgMatches) -> Result<(), String> {
    println!("Verifying contributions...");

    let path = Path::new(sub_matches.value_of("input").unwrap());
    let file =
        File::open(&path).map_err(|why| format!("Could not open `{}`: {}", path.display(), why))?;

    let reader = BufReader::new(file);
    let params = MPCParameters::read(reader, true)
        .map_err(|why| format!("Could not read `{}`: {}", path.display(), why))?;

    let radix_path = Path::new(sub_matches.value_of("radix-path").unwrap());
    let radix_file = File::open(radix_path)
        .map_err(|why| format!("Could not open `{}`: {}", radix_path.display(), why))?;

    let mut radix_reader = BufReader::with_capacity(1024 * 1024, radix_file);

    let circuit = Computation::without_witness(ir_prog);

    let result = params
        .verify(circuit, &mut radix_reader)
        .map_err(|_| "Verification failed".to_string())?;

    let contribution_count = result.len();
    println!(
        "\nTranscript contains {} contribution{}:",
        contribution_count,
        if contribution_count != 1 { "s" } else { "" }
    );

    for (i, hash) in result.iter().enumerate() {
        print!("{}: ", i);
        for b in hash.iter() {
            print!("{:02x}", b);
        }
        println!();
    }

    println!("\nContributions verified");
    Ok(())
}