use anyhow::{Context, Result};
use candid::Principal;
use clap::Parser;
use ic_agent::{Agent, AgentError};
use ic_utils::interfaces::management_canister::{builders::InstallCodeArgument, CanisterInstallMode, ManagementCanister};
use regex::Regex;
use std::fs;
use std::path::PathBuf;

/// CLI arguments
#[derive(Parser)]
#[command(about = "Measure cycles required by install_code for Wasm modules")] // about attribute
struct Opts {
    /// Canister ID to install code into
    #[arg(long)]
    canister_id: String,

    /// Replica URL (default http://localhost:4943)
    #[arg(long, default_value = "http://localhost:4943")]
    url: String,

    /// Wasm modules to install
    wasm: Vec<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();

    let agent = Agent::builder()
        .with_url(opts.url.clone())
        .build()
        .context("Failed to build agent")?;

    // In local development the root key must be fetched explicitly
    if opts.url.starts_with("http://localhost") {
        agent.fetch_root_key().await.context("fetch root key")?;
    }

    let canister_id = Principal::from_text(&opts.canister_id)?;
    let mgmt = ManagementCanister::create(&agent);

    let mut sizes = Vec::new();
    let mut cycles = Vec::new();

    for wasm_path in &opts.wasm {
        let wasm = fs::read(wasm_path)
            .with_context(|| format!("read wasm file {}", wasm_path.display()))?;
        let size = wasm.len() as f64;

        let arg = InstallCodeArgument {
            mode: CanisterInstallMode::Reinstall,
            canister_id,
            wasm_module: wasm,
            arg: Vec::new(),
        };

        // Send zero cycles to obtain the required cost from the error message
        let result: Result<(), AgentError> = mgmt
            .install_code(arg)
            .with_payment128(0)
            .call_and_wait()
            .await;

        let required = match result {
            Err(AgentError::CallError(ref ce)) => {
                parse_required_cycles(&ce.description)
                    .context("parse cycles from error message")?
            }
            Ok(()) => {
                eprintln!("install_code succeeded without cycles; cannot measure");
                continue;
            }
            Err(e) => return Err(e.into()),
        } as f64;

        println!(
            "{}: size={} bytes cycles={}",
            wasm_path.display(),
            size,
            required
        );
        sizes.push(size);
        cycles.push(required);
    }

    if !sizes.is_empty() {
        let (slope, intercept) = linear_regression(&sizes, &cycles)?;
        println!(
            "Linear approximation: cycles ~= {:.2} * bytes + {:.2}",
            slope, intercept
        );
    }

    Ok(())
}

/// Parse the cycle requirement from an install_code error message.
///
/// The management canister reports the number of cycles required when a call
/// is made with insufficient cycles. This function extracts that number from
/// the message.
fn parse_required_cycles(msg: &str) -> Result<u128> {
    let re = Regex::new(r"(\d+) cycles?").unwrap();
    if let Some(cap) = re.captures(msg) {
        let num = cap.get(1).unwrap().as_str().parse::<u128>()?;
        Ok(num)
    } else {
        Err(anyhow::anyhow!("could not find cycle amount in message: {}", msg))
    }
}

/// Perform a simple linear regression to fit y = a*x + b.
fn linear_regression(xs: &[f64], ys: &[f64]) -> Result<(f64, f64)> {
    if xs.len() != ys.len() {
        return Err(anyhow::anyhow!("input length mismatch"));
    }
    let n = xs.len();
    if n == 0 {
        return Err(anyhow::anyhow!("no data"));
    }

    let avg_x = xs.iter().sum::<f64>() / n as f64;
    let avg_y = ys.iter().sum::<f64>() / n as f64;
    let cov = xs
        .iter()
        .zip(ys.iter())
        .map(|(x, y)| (x - avg_x) * (y - avg_y))
        .sum::<f64>();
    let var = xs.iter().map(|x| (x - avg_x).powi(2)).sum::<f64>();
    if var == 0.0 {
        return Err(anyhow::anyhow!("zero variance"));
    }
    let slope = cov / var;
    let intercept = avg_y - slope * avg_x;
    Ok((slope, intercept))
}

