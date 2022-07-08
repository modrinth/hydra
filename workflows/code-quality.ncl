let toolchain = {
    toolchain = `nightly,
    override = true,
    profile = `minimal,
    components = `clippy,
} in {
  name = "Code Quality",
  on = {
     push = { branches = [ `master ] },
     pull_request = {},
  },
  env = {
    CARGO_TERM_COLOR = `always,
  },
  jobs = {
   lint = {
      runs-on = "ubuntu-latest",
      steps = [
        { uses = "actions/checkout@v2" },
        { name = "Cache Toolchain",
          id = `cache,
          uses = "actions/cache@v3",
          with = {
            path = m%"
              ~/.rustup
              ~/.cargo
            "%m,
            key = "${{ runner.os }}-lint-toolchain",
          },
        },
        { name = "Install toolchain",
          "if" = "${{ steps.cache.outputs.cache-hit == 'false' }}",
          uses = "actions-rs/toolchain@v1",
          with = toolchain,
        },
        { name = "Annotate with warnings",
          uses = "actions-rs/clippy-check@v1",
          with = {
            token = "${{ secrets.GITHUB_TOKEN }}",
            args = "--all-features",
          },
        },
      ]
    },
  }
}