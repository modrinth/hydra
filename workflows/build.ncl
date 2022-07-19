let nix = (fun command args => "nix %{command} -L %{args}") in
let registry = "ghcr.io" in
let username = "${{ github.repository_owner }}" in
let version = "${{ github.ref_name }}" in
let image_name = "hydra" in
{
  name = "Build",
  on = {
     push = {
         tags = [ "v*" ],
     },
  },
  env = {
    CARGO_TERM_COLOR = `always,
  },
  jobs = {
    build = {
      runs-on = `ubuntu-latest,
      permissions = {
        contents = `read,
        packages = `write,
      },
      steps = [
        { uses = "actions/checkout@v2" },
        { name = "Cache",
          uses = "actions/cache@v3",
          with = {
            path = m%"
              /nix/store
              /nix/var/nix/db/db.sqlite
              /nix/var/nix/profiles/per-user/${{ env.USER }}/profile/bin
              /nix/var/nix/profiles/default/bin
              /nix/var/nix/profiles/per-user/root/channels
            "%m,
            key = "${{ runner.os }}-nix-${{ hashFiles('./Cargo.lock') }}",
          },
        },
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
          run = (nix "build" ".#cross-hydra") ++ " && pbzip2 ./result/bin/hydra -c > hydra.bz2",
        },
        { name = "Publish Artifact",
          uses = "actions/upload-artifact@v3",
          with = {
            path = "hydra.bz2"
          },
        },

        { name = "Build Image",
          run = (nix "build" ".#docker-image"),
        },
        { name = "Login to Registry",
          run = "echo \"${{ secrets.GITHUB_TOKEN }}\" | docker login ghcr.io -u $ --password-stdin",
        },
        { name = "Publish Image",
          run = m%"
            IMAGE_ID="$(echo '%{registry}/%{username}/%{image_name}'| tr '[A-Z]' '[a-z]')"
            VERSION_TRIMMED="$(echo %{version} | sed 's/^v//')"
            docker load < ./result
            docker tag %{image_name} $IMAGE_ID:$VERSION_TRIMMED
            docker push $IMAGE_ID:$VERSION_TRIMMED
         "%m,
         }
      ]
    }
  }
}