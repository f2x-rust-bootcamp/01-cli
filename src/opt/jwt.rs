use super::verify_file;
use crate::{process_jwt_sign, process_jwt_verify, CmdExecutor};
use clap::Parser;
use enum_dispatch::enum_dispatch;

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExecutor)]
pub enum JwtSubCommand {
    #[command(about = "Sign a string")]
    Sign(JwtSignOpts),

    #[command(about = "Verify a signed string")]
    Verify(JwtVerifyOpts),
}

#[derive(Debug, Parser)]
pub struct JwtSignOpts {
    #[arg(long)]
    pub sub: String,

    #[arg(long)]
    pub aud: String,

    #[arg(long)]
    pub exp: String,

    #[arg(short, long)]
    pub key: String,
}

#[derive(Debug, Parser)]
pub struct JwtVerifyOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(short, long)]
    pub key: String,
}

///
/// impl CmdExecutor
///

impl CmdExecutor for JwtSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let token = process_jwt_sign(&self.sub, &self.aud, &self.exp, &self.key)?;
        println!("{:?}", token);
        Ok(())
    }
}

impl CmdExecutor for JwtVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let claims = process_jwt_verify(&self.input, &self.key)?;
        println!("{:?}", claims);
        Ok(())
    }
}
