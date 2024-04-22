mod opt;
mod process;
mod utils;

use enum_dispatch::enum_dispatch;
pub use opt::*;
pub use process::*;
pub use utils::*;

#[allow(async_fn_in_trait)]
#[enum_dispatch]
pub trait CmdExecutor {
    async fn execute(self) -> anyhow::Result<()>;
}
