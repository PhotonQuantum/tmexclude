#!/usr/bin/env nix-shell
#! nix-shell -i bash -p jq cargo-license nodejs nodePackages.pnpm

# This script collects the license information for all the dependencies

CARGO_LICENSES=$(cargo-license --manifest-path src-tauri/Cargo.toml --direct-deps-only --json | jq 'map(select(.license != null and .license != "") | {name, repository, license, version})|unique_by(.name)')
NPM_LICENSES=$(pnpx license-checker --direct --json | jq 'to_entries|map(select(.value.licenses != null and .value.licenses != "" and (.key | contains("tmexclude") | not) ) | {name:.key, repository:.value.repository, license:.value.licenses})')

cat <<EOF >src/licenses.ts
// noinspection AllyPlainJsInspection

export type License = {
  name: string;
  version?: string;
  repository: string | null;
  license: string;
};
export const cargoLicenses: Array<License> = $CARGO_LICENSES;
export const npmLicenses: Array<License> = $NPM_LICENSES;
EOF
