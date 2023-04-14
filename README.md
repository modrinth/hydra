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
    "refresh_token": <REFRESH TOKEN>,
    "expires_after": <SECONDS UNTIL EXPIRATION>
}
```
NOTE: If the user closes the tab without completing the flow, there is no way to detect it. Users should make sure that their apps can handle this case.

If any errors occur, a message of the following form will be sent and the socket will be closed:
```json
{"error": <ERROR MESSAGE>}
```

To refresh an existing token, send the following in a `POST` request to the `/refresh` endpoint:
```json
{
    "refresh_token": <REFRESH TOKEN HERE>
}
```

This does not need a socket or a webview, and the response is stored in the HTTP body. Otherwise, the response is the same (though you do need to update both tokens to the returned valuees).

## Deployment
To deploy Hydra, set the `HYDRA_CLIENT_ID` and `HYDRA_CLIENT_SECRET` environment variables using the values from an Azure application. You can build a release version of Hydra with `cargo build --release`, or download one of the artifacts on GitHub Actions. The local hostname uses the `HYDRA_HOST` and `HYDRA_PORT` variables, with the public hostname being stored in the `HYDRA_PUBLIC_URL` variable.
