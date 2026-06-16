# OneCLI Local qFido Setup

This folder is a local clone of `https://github.com/onecli/onecli`.

Use OneCLI according to the official docs:

- Quickstart: https://onecli.sh/docs/quickstart
- Coding agents: https://onecli.sh/docs/guides/coding-agents
- Gateway architecture: https://onecli.sh/docs/how-it-works
- MCP credential stubs: https://onecli.sh/docs/guides/credential-stubs/general-app

## Local Instance

- Dashboard: http://127.0.0.1:10254
- Gateway: `127.0.0.1:10255`
- Postgres host port: `127.0.0.1:15432`
- Docker services: `onecli` and `onecli-postgres-1`
- Local agent identifier: `qfido-operations`

Postgres uses host port `15432` because `5432` was already in use on this Mac.

## Start, Stop, And Inspect

Run these from this folder:

```bash
docker compose --env-file .env -f docker/docker-compose.yml up -d --wait
docker compose --env-file .env -f docker/docker-compose.yml ps
docker compose --env-file .env -f docker/docker-compose.yml logs -f
docker compose --env-file .env -f docker/docker-compose.yml down
```

Use `--env-file .env`; without it, Docker Compose may ignore the local port overrides.

Health check:

```bash
curl -fsS http://127.0.0.1:10255/healthz
```

## CLI

The OneCLI CLI is installed at:

```bash
/Users/thomasfellows/.local/bin/onecli
```

It is configured for the local dashboard:

```bash
/Users/thomasfellows/.local/bin/onecli config get api-host
/Users/thomasfellows/.local/bin/onecli auth status
```

Do not run `onecli agents list` in normal chat output because older local notes observed that it may print agent access tokens.

## Codex

Launch Codex through OneCLI exactly with the documented `onecli run` path:

```bash
/Users/thomasfellows/.local/bin/onecli run --agent qfido-operations -- codex
```

Preview what OneCLI would inject without launching Codex:

```bash
/Users/thomasfellows/.local/bin/onecli run --agent qfido-operations --dry-run -- codex
```

No custom `codex-oncli`, macOS app wrapper, LaunchAgent, Kubernetes allowlist refresh, or global `launchctl setenv` credential loader should be used for Codex. If Codex should use OneCLI, use `onecli run`. If Codex should not use OneCLI, open Codex normally.

## MCP Credential Stubs

For MCP servers that require credential files before they can start, follow the OneCLI credential-stub docs and use `onecli-managed` placeholders only in structurally valid stub files.

Do not place real API keys in project files. Add real credentials through the OneCLI dashboard or official OneCLI CLI/API flows, then let the gateway inject them at request time.
