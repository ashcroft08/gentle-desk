# Updating the app from inside the app (Tauri updater)

Today every new version is: download the installer, run it. This feature lets the app notice a new release, download it, verify it is ours, install it and restart.

## Specs

- S1 Ownership check. Updates are verified with the updater's own key pair: the public key is embedded in the app, the private key signs each release from CI and never lives in the repository. A release that does not verify is refused and nothing is installed.
- S2 Check on start, quietly. When the app opens it asks the endpoint for the newest version; if there is none newer, nothing is drawn and nothing is said. A failed check is silent (no network, no release) and never blocks the window.
- S3 One notice, one action. When a newer version exists a single line says so with its version and offers "Install and restart"; while it downloads it shows progress; a failure says so and offers to try again. It can be dismissed for this run.
- S4 Restart only after the install. The app restarts itself once the update is installed, so the running process is never the old one pretending to be new.
- S5 The release carries what the updater needs: the signed artifacts and a `latest.json` manifest pointing at this release's installer and its signature, built by the same workflow that builds the installer.
- S6 Nothing here publishes on its own: the workflow only runs on a tag or by hand, as today.

## Tasks

- [ ] T1 (S1, S5) Release side: `createUpdaterArtifacts`, signing with the CI secrets, and the `latest.json` manifest attached to the release.
- [ ] T2 (S1, S2) App side: the plugin, the public key and the endpoint in the config.
- [ ] T3 (S2, S3, S4) The notice: a pure state machine (idle, checking, available, downloading, ready, failed, dismissed) with tests, and its component.
- [ ] T4 (S7) Real check: publish a version, install the previous one, and watch it offer and install the new one. Owner: the user, because it needs two published releases.

## Log

- L1 (2026-11-11, user): "incluir el plugin de updater de Tauri lo podemos hacer?" and then "continuemos con el updater", after closing 0.1.1 first.
