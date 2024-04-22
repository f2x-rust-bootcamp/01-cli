mod opt;
mod process;
mod utils;

pub use opt::{
    Base64SubCommand, HttpSubCommand, JwtSubCommand, Opts, SubCommand, TextSignFormat,
    TextSubCommand,
};
pub use process::*;
pub use utils::*;

#[allow(async_fn_in_trait)]
pub trait CmdExecutor {
    async fn execute(self) -> anyhow::Result<()>;
}
