let nix = (fun command args => "nix %{command} -L %{args}") in
{
  name = "Build",
  on = {
     push = { branches = [ `master ] },
     pull_request = {},
  },
  env = {
    CARGO_TERM_COLOR = `always,
  },
  jobs = {
    build = {
      runs-on = `ubuntu-latest,
      steps = [
        { uses = "actions/checkout@v2" },
        { uses = "cachix/install-nix-action@v17",
          with = {
            extra_nix_config = m%"
              access-tokens = github.com=${{ secrets.GITHUB_TOKEN }}
              trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=
              substituters = https://cache.nixos.org/"%m,
          },
        },
        { name = "Install Dependencies",
          run = nix "profile install" "nixpkgs#pbzip2",
        },
        { name = "Build Hydra",
          run = (nix "build" "") ++ " && pbzip2 ./result/bin/hydra -c > hydra.bz2",
        },
        { name = "Build Image",
          run = (nix "build" ".#docker-image") ++ " && cp result hydra-docker.tar.gz",
        },
        { name = "Publish Artifacts",
          uses = "actions/upload-artifact@v3",
          with = {
            path = m%"
              hydra.bz2
              hydra-docker.tar.gz
            "%m
          },
        }
      ]
    }
  }
}