use clap::Parser;
use obot_converter::{
    cli::Cli,
    formats::{
        mhr::MHRReplay,
        omegabot::OmegabotReplay,
        replay::{Replay, ReplayFormat},
        tasbot::TasbotReplay,
    },
};

fn main() -> color_eyre::Result<()> {
    let args = Cli::parse();

    println!("Converting {} to intermediate...", args.from_fmt);
    let replay: Replay = match args.from_fmt.as_str() {
        "tasbot" => TasbotReplay::load(&args.from)?.to_universal()?,
        "omegabot" => OmegabotReplay::load(&args.from)?.to_universal()?,
        "mhr" => MHRReplay::load(&args.from)?.to_universal()?,
        _ => unreachable!(),
    };

    println!(
        "Converting intermediate to {} (this may take a while)...",
        args.to_fmt
    );
    match args.to_fmt.as_str() {
        "tasbot" => TasbotReplay::from_universal(replay)?.save(&args.to)?,
        "omegabot" => OmegabotReplay::from_universal(replay)?.save(&args.to)?,
        "mhr" => MHRReplay::from_universal(replay)?.save(&args.to)?,
        _ => unreachable!(),
    };

    Ok(())
}
