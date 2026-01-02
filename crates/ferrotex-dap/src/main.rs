use anyhow::Result;

fn main() -> Result<()> {
    #[cfg(feature = "tectonic-engine")]
    {
        ferrotex_dap::run_tectonic_session()?;
    }
    #[cfg(not(feature = "tectonic-engine"))]
    {
        ferrotex_dap::run_mock_session()?;
    }
    Ok(())
}
