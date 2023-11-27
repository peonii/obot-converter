use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(short, long)]
    pub from: String,

    #[clap(short, long)]
    pub from_fmt: String,

    #[clap(short, long)]
    pub to: String,

    #[clap(short, long)]
    pub to_fmt: String,
}
