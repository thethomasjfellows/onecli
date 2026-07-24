# qFido OneCLI Production Runbook

This is the operational source of truth for qFido's OneCLI fork. Context systems may link here, but setup steps, release controls, and recovery procedures stay in this repository.

## Repository ownership

- `upstream` fetches `https://github.com/onecli/onecli` and has no usable push URL.
- `origin` fetches and pushes `https://github.com/thethomasjfellows/onecli`.
- `main` mirrors upstream source.
- `qfido/production` owns qFido packaging, operations, and approved custom changes.
- Feature branches intended for upstream must start from `upstream/main` and contain no qFido-only files.

Before integrating upstream changes, create a recoverable local reference, fetch both remotes, inspect the full comparison, and run the repository quality gates. Merge into `qfido/production` only after resolving the review deliberately.

## Required quality gates

From the repository root:

```bash
pnpm install --frozen-lockfile
pnpm check
pnpm test
pnpm build
```

Do not publish an image from a dirty worktree or a commit that has not passed these gates.

## Publish an immutable image

Run the `Publish qFido image` workflow from `qfido/production`. Supply a new label in the form `qfido-YYYYMMDD-N`.

The workflow:

- builds native Linux AMD64 and ARM64 images;
- refuses to replace an existing release label;
- publishes no `latest` tag;
- creates one multi-architecture manifest in `ghcr.io/thethomasjfellows/onecli`;
- records the source commit and final manifest digest as a workflow artifact and job summary.

Copy the resulting digest reference exactly:

```text
ghcr.io/thethomasjfellows/onecli@sha256:...
```

The digest, not the release label, is the deployable identity.

## Database protection gate

Before changing the running image:

1. Identify the exact Compose project and Postgres volume used by the running instance.
2. Create a timestamped PostgreSQL logical backup outside the container and record its checksum.
3. Restore that backup into an isolated temporary PostgreSQL instance.
4. Verify that the restored database opens and contains the expected OneCLI schema.
5. Record the current image digest and keep it available for rollback.

If the backup cannot be restored, stop. Do not deploy.

## Deploy

Set `ONECLI_IMAGE` in the local environment file to the approved digest reference. The Compose file accepts this full override while retaining the upstream image as a development fallback.

Review the resolved Compose configuration before applying it:

```bash
docker compose --env-file .env -f docker/docker-compose.yml config
docker compose --env-file .env -f docker/docker-compose.yml pull onecli
docker compose --env-file .env -f docker/docker-compose.yml up -d --wait onecli
```

Verify:

- the dashboard loads;
- `/healthz` succeeds on the gateway;
- the running container reports the intended digest;
- an existing connection still works;
- the self-managed Airbyte connection can mint a token and call a read-only Public API endpoint through the gateway.

Do not expose secret values or agent tokens in logs or chat output.

## Roll back

Set `ONECLI_IMAGE` back to the previously recorded digest and recreate only the OneCLI service. Recheck the dashboard and gateway health.

If the application changed the database incompatibly, stop the application, restore the verified pre-deploy backup into an isolated replacement database or volume, and switch only after the restored instance passes validation. Do not overwrite the only production database copy.

## Upstream contribution flow

Keep upstream contributions on clean feature branches based on `upstream/main`. Open an upstream issue before a feature PR, follow the repository template, and let the commit and PR author fields provide contributor credit.

After upstream merges a contribution:

1. fetch `upstream`;
2. update the fork's `main` to the reviewed upstream commit;
3. merge the updated `main` into `qfido/production`;
4. rerun every quality gate;
5. publish and deploy a new immutable digest through this runbook.

Never copy operational instructions back into OpenKnowledge. OpenKnowledge should contain only durable context, ownership boundaries, and links to this runbook.
