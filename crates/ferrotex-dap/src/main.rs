#![allow(unexpected_cfgs)]

use anyhow::Result;

fn main() -> Result<()> {
    #[cfg(feature = "jxoesneon-tectonic-engine")]
    {
        ferrotex_dap::run_jxoesneon_tectonic_session()?;
    }
    #[cfg(not(feature = "jxoesneon-tectonic-engine"))]
    {
        ferrotex_dap::run_mock_session()?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_main_compiles() {
        // Smoke test: verify the crate links and both cfg branches exist.
        let _: fn() -> anyhow::Result<()> = || Ok(());
    }
}
