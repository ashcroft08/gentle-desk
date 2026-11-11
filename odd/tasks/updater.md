# Updating the app from inside the app (Tauri updater)

Today every new version is: download the installer, run it. This feature lets the app notice a new release, download it, verify it is ours, install it and restart.

## Specs

- S1 Ownership check. Updates are verified with the updater's own key pair: the public key is embedded in the app, the private key signs each release from CI and never lives in the repository. A release that does not verify is refused and nothing is installed.
- S2 Check on start, quietly. When the app opens it asks the endpoint for the newest version; if there is none newer, nothing is drawn and nothing is said. A failed check is silent (no network, no release) and never blocks the window.
- S3 One notice, one action. When a newer version exists a single line says so with its version and offers "Install and restart"; while it downloads it shows progress; a failure says so and offers to try again. It can be dismissed for this run.
- S4 Honest about the restart. On Windows the updater closes the app itself to hand over to the installer (plugin source and Tauri docs), so the line says "The app will close while it installs" BEFORE the button is pressed and the app never calls a restart of its own. Whether the new version opens by itself is confirmed only in the real cycle (T4); until then nothing promises it.
- S5 The release carries what the updater needs: the signed artifacts and a `latest.json` manifest pointing at this release's installer and its signature, built by the same workflow that builds the installer.
- S7 A build without the signing key must still work: `createUpdaterArtifacts` stays off in the repository config (with it on and no private key `tauri build` exits 1), and the release workflow turns it on only when the signing secret exists.
- S6 Nothing here publishes on its own: the workflow only runs on a tag or by hand, as today.

## Tasks

- [ ] T1 (S1, S5, S7) Release side: turn `createUpdaterArtifacts` on from the workflow only with the CI secrets, sign, and attach the `latest.json` manifest to the release.
- [x] T2 (S1, S2) App side: the plugin, the endpoint in the config, and the check that stays silent while the key is still a placeholder. The real public key is now in the config.
- [x] T3 (S2, S3, S4) The notice: a pure state machine with 6 tests and its component with 6 more, wired into App.svelte.
- [ ] T4 (S4) Real check: publish a version, install the previous one, and watch it offer and install the new one. Owner: the user, because it needs two published releases.

## Log

- L1 (2026-11-11, user): "incluir el plugin de updater de Tauri lo podemos hacer?" and then "continuemos con el updater", after closing 0.1.1 first.
- L2 (2026-11-11, verification the user asked for): three defects found and handled. A build without the private key failed (S7, createUpdaterArtifacts now off); the app's own relaunch was unreachable on Windows because the plugin exits the process before installing (S4 rewritten, relaunch and the process plugin removed); and the installer grew from about 2.4 MiB to 3.40 MiB with the updater plugin. The silent start with the placeholder key was checked in the built exe: it opens, stays alive and draws no update line.
- L3 (2026-11-11, user authorized): the user authorized reading C:/Users/josep/.tauri/gentle-desk.key.pub, and that file already holds the base64 form Tauri v2 expects. Only that public value was pasted into the config; the private key and its password never enter the repository.
- L4 (2026-11-11, agent): the native review of the updater (lineage review-3129dee0591f0892, three lenses plus a refuter) raised one critical finding, R3-release-artifact, claiming the NSIS updater artifact is a `.nsis.zip` and that the release staging therefore wrote a broken `latest.json`; the refuter corroborated it and one bounded correction was required. The targeted validator then rejected that correction while repeating the same premise, so the lineage ended escalated; the official Tauri v2 updater documentation contradicts the premise and a real signed build settled it: run 42941461737 (workflow_dispatch on feat/updater) printed "update signing: ON" and produced `Gentle Shell Desktop_0.1.1_x64-setup.exe.sig` plus a `latest.json` whose signature is byte-identical to that .sig and whose url points at the installer, and no .nsis.zip was produced at all. The correction stands on its own merit: the staging now fails the build when a signing key is set and no signed artifact was found, instead of silently writing signed=false.
