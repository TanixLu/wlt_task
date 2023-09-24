mod config;
mod email;
mod log;
mod utils;
mod wlt_api;

use crate::config::Config;
use crate::log::log;
use crate::utils::AnyResult;

fn main() -> AnyResult<()> {
    let main_try = || -> AnyResult<()> {
        let config = Config::load()?;

        Ok(())
    };

    if let Err(e) = main_try() {
        log(e.to_string())?;
    }

    Ok(())
}
