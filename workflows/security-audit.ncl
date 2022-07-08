let audited-files = { paths = ["**/Cargo.toml", "**/Cargo.lock"] } in
{
  name = "Security Audit",
  on = {
    push = audited-files,
    pull_request = audited-files,
  },
  env = {
    CARGO_TERM_COLOR = `always,
  },
  jobs = {
    audit = {
      runs-on = `ubuntu-latest,
      steps = [
        { uses = "actions/checkout@v1" },
        { name = "Cache Toolchain",
          uses = "actions/cache@v3",
          id = `cache,
          with = {
            path = m%"
              ~/.rustup
              ~/.cargo
            "%m
          },
        },
        { name = "Install Toolchain",
          "if" = "${{ steps.cache.outputs.cache-hit == 'false' }}",
          uses = "actions-rs/toolchain@v1",
          with = {
            toolchain = `nightly,
            override = true,
            profile = `minimal,
          },
        },
        { uses = "actions-rs/audit-check@v1",
          with = {
            token = "${{ secrets.GITHUB_TOKEN }}",
          },
        },
      ],
    },
  },
}