# Hydra - Minecraft Microsoft Authentication Flow
![GitHub branch checks state](https://img.shields.io/github/checks-status/modrinth/hydra/master)
![GitHub license](https://img.shields.io/github/license/modrinth/hydra)
![Discord](https://img.shields.io/discord/734077874708938864)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/modrinth/hydra)
![GitHub issues](https://img.shields.io/github/issues/modrinth/hydra)

## Introduction
Hydra is a simple wrapper around the Microsoft authentication flow intended to provide a way for launcher developers to authenticate accounts without needing to compile client-side secrets in code.

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

If any errors occur, a message of the following form will be sent and the socket will be closed:
```json
{"error": <ERROR MESSAGE>}
```
