# Hydra - Minecraft Microsoft Authentication Flow
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/modrinth/hydra/Code%20Quality)
![GitHub license](https://img.shields.io/github/license/modrinth/hydra?)
![Discord](https://img.shields.io/discord/734077874708938864)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/modrinth/hydra)
![GitHub issues](https://img.shields.io/github/issues/modrinth/hydra)

## Introduction
Hydra is a simple wrapper around the Microsoft authentication flow intended to provide a way for launcher developers to authenticate accounts without needing to compile in a web server, create a (possibly impersonatable) Azure app, or deal with the headaches that MSA is often known to cause.

## Usage
Hydra is based on a web socket, and to begin you will need to connect to one using the `/` route. You will then receive a message of the following format:
```json
{"login_code": <LOGIN CODE>}
```

This contains the UUID needed to use the flow. While the code should be the same per computer, this route must be called in order to have the socket open.

Next, you will need to open a web browser to the `/login` route with the `id` query parameter set to the login code you just got. Once the user signs in, a message with the following format will be sent over the socket:

```json
{
    "token": <BEARER TOKEN>,
    "expires": <SECONDS UNTIL EXPIRATION>
}
```
NOTE: If the user closes the tab without completing the flow, there is no way to detect it. Users should make sure that their apps can handle this case.

If any errors occur, a message of the following form will be sent and the socket will be closed:
```json
{"error": <ERROR MESSAGE>}
```

As a convenience, the APIs provided via `api.minecraftservices.com` can be accessed using the `/services` redirect endpoint.


## Deployment
To deploy Hydra, set the `HYDRA_CLIENT_ID` and `HYDRA_CLIENT_SECRET` environment variables using the values from an Azure application. You can build a release version of Hydra with `cargo build --release`, or download one of the artifacts on GitHub Actions. Alternatively, you can use the `.#hydra` and `.#cross-hydra` (builds static GNU/Linux executables using Musl) attributes in Nix. The local hostname uses the `HYDRA_HOST` and `HYDRA_PORT` variables, with the public hostname being stored in the `PUBLIC_URL` variable.

If using the `tls` feature, you will need to either set the `HYDRA_CERT` and `HYDRA_KEY` variables to your TLS certificate and key paths or place them in the `$XDG_CONFIG_DIR/hydra` directory.

Docker images can also be created. In additon to the ones uploaded to GitHub actions, you can use the flake manually should you want to use a customized feature set. To build an image, use the `.#docker-image` flake attribute.

As a convenience, a `cargo make` script is included to build images with the given certificates quickly. Using `cargo make tls-image <CERT_PATH> <KEY_PATH>` will create a Docker image with the given TLS certificates installed.
