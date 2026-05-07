use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, help = "Output requests headers")]
    pub(crate) header: bool,
}
