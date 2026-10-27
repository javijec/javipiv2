#!/usr/bin/env bash
set -Eeuo pipefail

usage() {
  cat <<'HELP'
JaviPiV2 - bootstrap experimental (solo preflight, sin instalación)

Uso:
  bash installer/bootstrap.sh --help
  bash installer/bootstrap.sh --check
  bash installer/bootstrap.sh --plan minimal|developer|full

No ejecutar con sudo. La instalación real todavía NO está implementada.
HELP
}

check_host() {
  printf 'Sistema: %s\n' "$(uname -s)"
  printf 'Arquitectura: %s\n' "$(uname -m)"
  if [[ -r /etc/os-release ]]; then
    # Leer metadatos del sistema no modifica el host.
    # shellcheck disable=SC1091
    source /etc/os-release
    printf 'Distribución: %s\n' "${PRETTY_NAME:-desconocida}"
  fi
  printf 'Nota: preflight informativo; no instala dependencias ni toca /srv.\n'
}

plan_profile() {
  local profile=$1
  case "$profile" in
    minimal|developer|full) ;;
    *) printf 'Perfil desconocido: %s\n' "$profile" >&2; return 2 ;;
  esac
  local root profile_file catalog module
  root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
  profile_file="$root/profiles/$profile.list"
  catalog="$root/catalog/modules.tsv"
  [[ -f "$profile_file" && -f "$catalog" ]] || {
    printf 'Faltan archivos de catálogo/perfil.\n' >&2
    return 2
  }
  printf 'Perfil %s (PREVIEW: no instala ni habilita nada):\n' "$profile"
  while IFS= read -r module || [[ -n "$module" ]]; do
    [[ -z "$module" || "$module" == \#* ]] && continue
    [[ "$module" =~ ^[a-z0-9-]+$ ]] || { printf 'Módulo inválido: %s\n' "$module" >&2; return 2; }
    awk -F '\t' -v id="$module" '$1 == id {ok=1} END {exit !ok}' "$catalog" || {
      printf 'Módulo no registrado: %s\n' "$module" >&2
      return 2
    }
    printf '  - %s\n' "$module"
  done < "$profile_file"
  printf 'La instalación real permanece BLOQUEADA hasta su implementación y pruebas.\n'
}

case "$#:${1:-}" in
  0:*) usage ;;
  1:--help|1:-h) usage ;;
  1:--check) check_host ;;
  2:--plan) plan_profile "$2" ;;
  *) usage >&2; exit 2 ;;
esac
