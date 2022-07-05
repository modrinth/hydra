# Hydra - Minecraft Microsoft Authentication Flow
![GitHub branch checks state](https://img.shields.io/github/checks-status/modrinth/hydra/master)
![GitHub license](https://img.shields.io/github/license/modrinth/hydra)
![Discord](https://img.shields.io/discord/734077874708938864)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/modrinth/hydra)
![GitHub issues](https://img.shields.io/github/issues/modrinth/hydra)

## Introduction
Hydra is a simple wrapper around the Microsoft authentication flow intended to provide a way for launcher developers to authenticate accounts without needing to compile client-side secrets in code.

## Usage
To get a Minecraft bearer token, open a browser to the `/login` route and allow the user to sign into their Microsoft account. The response is JSON encoded and has the following format:
```json
{
    "token": <BEARER TOKEN>,
    "expires": <TIME IN SECONDS UNTIL THE TOKEN EXPIRES>,
    "flow_done": true
}
```

The sole purpose of the `flow_done` field is to be a sentinel which makes it easier to distinguish the successful login from any other response. Errors should be handled by the application as they are received.
