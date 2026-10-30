# Backups-B passive status
Do not read credentials or protected configuration; preserve unrelated changes.

## Specs
S1. "Revisá las fuentes existentes de Restic en JaviPi original y reutilizá Backups-A para incorporar `backup status` en CLI y posteriormente TUI."
S2. "Priorizá información real: configuración detectable, snapshots, último backup y resultado del último intento. Diferenciá claramente desconocido, no configurado y sin snapshots; nunca deduzcas éxito de un snapshot existente."
S3. "No ejecutes scripts privilegiados, no leas credenciales ni modifiques backups. Si no existe una fuente segura para algún dato, mostr á desconocido y documentá qué falta, sin inventar información."
S4. "Mantené el código simple y modular. Validá en ARM64, creá un commit local y conservá los cambios ajenos."
S5. "Sin sudo, push, modificaciones productivas ni iniciar otra etapa."

## Tasks
- T1 [S1-S3,S5] done: original helper conflates missing configuration and empty JSON; no trusted unprivileged attempt source, see L2
- T2 [S1-S3,S5] done: current status unknown; optional stdin displayed separately via Backups-A; TUI manual view; commit: pending
- T3 [S1-S5] done: ARM64 181 passed, 1 ignored; fmt/check/Clippy/build passed with pre-existing F1 changes present; commit: closing work unit

## Log
L4. ARM64 full suite 181 passed, 0 failed, 1 ignored, fmt/check/strict Clippy/build passed. The suite included the pre-existing uncommitted F1 changes and they are not part of Backups-B commit. Prior combined focused command timed out during TUI compilation; the later independently requested full suite passed. Actual CLI status run showed all unknown without host queries. Backups TUI synthetic validation only; no manual validation. No live source found for actual configuration, empty repository, last backup or last attempt; these remain unavailable and unknown, not asserted absent.
L3. CLI status has no safe live source and returns unknown; stdin JSON is explicitly supplied evidence, not current repository/configuration/last attempt. Empty supplied list does not prove an empty repo. TUI Backups is manual and pure. Four CLI focused tests passed, TUI RED observed. A combined focused command timed out during compilation after CLI tests; no TUI tests had begun. Full ARM64 suite started independently. Do not stage pre-existing F1 changes.
L2. Original `javipi-backup snapshots --json` emits `[]` when config missing and calls Restic when present; dashboard checks existence, not configuration validity. Helper may require credentials/privilege and repository access, so do not execute it or inspect protected config. No safe source for last-attempt status found. Backups-A parses caller-supplied snapshots; these are evidence of supplied data, not proof of current configuration, repository, or latest attempt.
L1. JaviPiV2 — Implementá Backups-B.

Revisá las fuentes existentes de Restic en JaviPi original y reutilizá Backups-A para incorporar `backup status` en CLI y posteriormente TUI.

Priorizá información real: configuración detectable, snapshots, último backup y resultado del último intento. Diferenciá claramente desconocido, no configurado y sin snapshots; nunca deduzcas éxito de un snapshot existente.

No ejecutes scripts privilegiados, no leas credenciales ni modifiques backups. Si no existe una fuente segura para algún dato, mostr á desconocido y documentá qué falta, sin inventar información.

Mantené el código simple y modular. Validá en ARM64, creá un commit local y conservá los cambios ajenos.

Sin sudo, push, modificaciones productivas ni iniciar otra etapa.
