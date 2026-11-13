use clap::Parser;

#[derive(Parser)]
#[command()]
pub struct Args {
    #[arg(long)]
    pub x11: bool,
}

pub fn parse() -> Args {
    Args::parse()
}
