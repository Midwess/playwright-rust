# Versioned landing site (playwright-rust.dev)

The landing page is published per-version so a visitor on any release sees the
site as it shipped, with a dropdown to switch versions and a banner when they
are not on the latest stable.

## Layout (on the `gh-pages` branch)

```
/                 root redirect → newest /vX.Y.Z/
/versions.json    { "latest": "X.Y.Z", "versions": ["X.Y.Z", … newest-first] }
/v0.14.0/         immutable release snapshot
/v0.15.0/         …
/dev/             main HEAD (unreleased preview)
/CNAME, /.nojekyll
```

The SPA reads `/versions.json` at runtime to populate the dropdown (so an old
snapshot can still list versions released after it was built) and compares its
own build-time `SITE_VERSION` to `latest` to decide whether to show the
"newer release available" / "unreleased dev build" banner.

## How a build knows its version

`crates/site/build.rs` reads the `SITE_VERSION` env var and bakes it in via
`env!("SITE_VERSION")`. The deploy sets it: `dev` for the main-HEAD build, the
release version (e.g. `0.14.0`) for a snapshot. Each snapshot is built with
`trunk build --public-url /<dest>/` so its assets resolve under the subpath.

## Deploy ([.github/workflows/pages.yml](../.github/workflows/pages.yml))

One job: lint the site crates, run the playwright-rs **dogfood gate** (build a
root-served `SITE_VERSION=dev` build and drive it with the binding — the deploy
only proceeds if it passes), then build the target snapshot, drop it into the
`gh-pages` worktree under `/<dest>/`, regenerate `versions.json` + the root
redirect ([deploy/update-manifest.sh](../crates/site/deploy/update-manifest.sh)),
and commit.

Triggers:
- **push to `main`** (site paths) → rebuilds `/dev/`.
- **`workflow_dispatch` with `version=X.Y.Z`** → publishes `/vX.Y.Z/` from
  current source.

## One-time bootstrap

1. Run the workflow manually with `version=0.14.0` (builds the current source —
   which already represents the 0.14.0 release and includes the switcher — as
   `/v0.14.0/`). This creates `gh-pages` with `/v0.14.0/`, `versions.json`,
   and the root redirect.
2. Push any site change to `main` (or run with a blank version) to publish
   `/dev/`.
3. **Repo setting (manual):** Settings → Pages → Build and deployment →
   Source: **Deploy from a branch** → Branch: **`gh-pages` / (root)**. This
   switches off the old single-artifact deploy.

## Per release

After tagging/publishing `vX.Y.Z`, publish its snapshot from the tagged source:

```
gh workflow run pages.yml -f version=X.Y.Z --ref vX.Y.Z
```

This builds `/vX.Y.Z/`, makes it the new `latest` (root redirect + manifest),
and leaves older snapshots untouched. Worth adding to the release runbook
(`.claude/skills/release-process`).
