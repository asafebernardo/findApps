#!/usr/bin/env bash
# Snapcraft breaks on project paths that contain spaces (e.g. "Área de trabalho").
# This script copies the tree to a clean path, packs the snap, and copies it back.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUILD_DIR="${FINDAPPS_SNAP_BUILD_DIR:-$HOME/findapps-snap-build}"
MODE="${1:---destructive-mode}"

echo "==> Syncing project to ${BUILD_DIR}"
mkdir -p "${BUILD_DIR}"
rsync -a --delete \
  --exclude target \
  --exclude parts \
  --exclude stage \
  --exclude prime \
  --exclude '*.snap' \
  --exclude .git \
  "${ROOT}/" "${BUILD_DIR}/"

echo "==> Running: snapcraft pack ${MODE}"
cd "${BUILD_DIR}"
if [[ "${MODE}" == "--destructive-mode" ]]; then
  sudo snapcraft pack --destructive-mode
else
  snapcraft pack ${MODE}
fi

SNAP_FILE="$(ls -1t "${BUILD_DIR}"/findapps_*.snap | head -n1)"
echo "==> Copying $(basename "${SNAP_FILE}") back to project"
cp -f "${SNAP_FILE}" "${ROOT}/"
echo "Done: ${ROOT}/$(basename "${SNAP_FILE}")"
echo "Install with:"
echo "  sudo snap install --dangerous --classic \"${ROOT}/$(basename "${SNAP_FILE}")\""
