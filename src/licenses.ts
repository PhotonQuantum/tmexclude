export type License = {
  name: string;
  version?: string;
  repository: string | null;
  license: string;
};
export const cargoLicenses: Array<License> = [
  {
    "name": "arc-swap",
    "repository": "https://github.com/vorner/arc-swap",
    "license": "Apache-2.0 OR MIT",
    "version": "1.5.1"
  },
  {
    "name": "assert_cmd",
    "repository": "https://github.com/assert-rs/assert_cmd.git",
    "license": "Apache-2.0 OR MIT",
    "version": "2.0.6"
  },
  {
    "name": "cocoa",
    "repository": "https://github.com/servo/core-foundation-rs",
    "license": "Apache-2.0 OR MIT",
    "version": "0.24.1"
  },
  {
    "name": "core-foundation",
    "repository": "https://github.com/servo/core-foundation-rs",
    "license": "Apache-2.0 OR MIT",
    "version": "0.9.3"
  },
  {
    "name": "crossbeam",
    "repository": "https://github.com/crossbeam-rs/crossbeam",
    "license": "Apache-2.0 OR MIT",
    "version": "0.8.2"
  },
  {
    "name": "directories",
    "repository": "https://github.com/soc/directories-rs",
    "license": "Apache-2.0 OR MIT",
    "version": "4.0.1"
  },
  {
    "name": "eyre",
    "repository": "https://github.com/yaahc/eyre",
    "license": "Apache-2.0 OR MIT",
    "version": "0.6.8"
  },
  {
    "name": "fsevent-stream",
    "repository": "https://github.com/PhotonQuantum/fsevent-stream",
    "license": "MIT",
    "version": "0.2.3"
  },
  {
    "name": "futures",
    "repository": "https://github.com/rust-lang/futures-rs",
    "license": "Apache-2.0 OR MIT",
    "version": "0.3.25"
  },
  {
    "name": "itertools",
    "repository": "https://github.com/rust-itertools/itertools",
    "license": "Apache-2.0 OR MIT",
    "version": "0.10.5"
  },
  {
    "name": "jwalk",
    "repository": "https://github.com/jessegrosjean/jwalk",
    "license": "MIT",
    "version": "0.6.0"
  },
  {
    "name": "log",
    "repository": "https://github.com/rust-lang/log",
    "license": "Apache-2.0 OR MIT",
    "version": "0.4.17"
  },
  {
    "name": "maplit",
    "repository": "https://github.com/bluss/maplit",
    "license": "Apache-2.0 OR MIT",
    "version": "1.0.2"
  },
  {
    "name": "moka",
    "repository": "https://github.com/moka-rs/moka",
    "license": "Apache-2.0 OR MIT",
    "version": "0.9.6"
  },
  {
    "name": "objc",
    "repository": "http://github.com/SSheldon/rust-objc",
    "license": "MIT",
    "version": "0.2.7"
  },
  {
    "name": "parking_lot",
    "repository": "https://github.com/Amanieu/parking_lot",
    "license": "Apache-2.0 OR MIT",
    "version": "0.12.1"
  },
  {
    "name": "serde",
    "repository": "https://github.com/serde-rs/serde",
    "license": "Apache-2.0 OR MIT",
    "version": "1.0.148"
  },
  {
    "name": "serde_json",
    "repository": "https://github.com/serde-rs/json",
    "license": "Apache-2.0 OR MIT",
    "version": "1.0.89"
  },
  {
    "name": "serde_yaml",
    "repository": "https://github.com/dtolnay/serde-yaml",
    "license": "Apache-2.0 OR MIT",
    "version": "0.9.14"
  },
  {
    "name": "shellexpand",
    "repository": "https://gitlab.com/ijackson/rust-shellexpand",
    "license": "Apache-2.0 OR MIT",
    "version": "2.1.2"
  },
  {
    "name": "tap",
    "repository": "https://github.com/myrrlyn/tap",
    "license": "MIT",
    "version": "1.0.1"
  },
  {
    "name": "tauri",
    "repository": "https://github.com/tauri-apps/tauri",
    "license": "Apache-2.0 OR MIT",
    "version": "1.2.1"
  },
  {
    "name": "tauri-build",
    "repository": "https://github.com/tauri-apps/tauri/tree/dev/core/tauri-build",
    "license": "Apache-2.0 OR MIT",
    "version": "1.2.1"
  },
  {
    "name": "tempfile",
    "repository": "https://github.com/Stebalien/tempfile",
    "license": "Apache-2.0 OR MIT",
    "version": "3.3.0"
  },
  {
    "name": "thiserror",
    "repository": "https://github.com/dtolnay/thiserror",
    "license": "Apache-2.0 OR MIT",
    "version": "1.0.37"
  },
  {
    "name": "ts-rs",
    "repository": "https://github.com/Aleph-Alpha/ts-rs",
    "license": "MIT",
    "version": "6.2.0"
  },
  {
    "name": "vergen",
    "repository": "https://github.com/rustyhorde/vergen",
    "license": "Apache-2.0 OR MIT",
    "version": "7.4.3"
  },
  {
    "name": "window-vibrancy",
    "repository": "https://github.com/tauri-apps/tauri-plugin-vibrancy",
    "license": "Apache-2.0 OR MIT",
    "version": "0.3.2"
  },
  {
    "name": "xattr",
    "repository": "https://github.com/Stebalien/xattr",
    "license": "Apache-2.0 OR MIT",
    "version": "0.2.3"
  }
];
export const npmLicenses: Array<License> = [
  {
    "name": "@emotion/react@11.10.5",
    "repository": "https://github.com/emotion-js/emotion/tree/main/packages/react",
    "license": "MIT"
  },
  {
    "name": "@emotion/server@11.10.0",
    "repository": "https://github.com/emotion-js/emotion/tree/main/packages/server",
    "license": "MIT"
  },
  {
    "name": "@emotion/styled@11.10.5",
    "repository": "https://github.com/emotion-js/emotion/tree/main/packages/styled",
    "license": "MIT"
  },
  {
    "name": "@mantine/core@5.9.2",
    "repository": "https://github.com/mantinedev/mantine",
    "license": "MIT"
  },
  {
    "name": "@mantine/hooks@5.9.2",
    "repository": "https://github.com/mantinedev/mantine",
    "license": "MIT"
  },
  {
    "name": "@mantine/notifications@5.9.2",
    "repository": "https://github.com/mantinedev/mantine",
    "license": "MIT"
  },
  {
    "name": "@mantine/utils@5.9.2",
    "repository": "https://github.com/mantinedev/mantine",
    "license": "MIT"
  },
  {
    "name": "@tabler/icons@1.115.0",
    "repository": "https://github.com/tabler/tabler-icons",
    "license": "MIT"
  },
  {
    "name": "@tauri-apps/api@1.2.0",
    "repository": "https://github.com/tauri-apps/tauri",
    "license": "Apache-2.0 OR MIT"
  },
  {
    "name": "@tauri-apps/cli@1.2.1",
    "repository": "https://github.com/tauri-apps/tauri",
    "license": "Apache-2.0 OR MIT"
  },
  {
    "name": "@types/lodash@4.14.191",
    "repository": "https://github.com/DefinitelyTyped/DefinitelyTyped",
    "license": "MIT"
  },
  {
    "name": "@types/node@18.11.11",
    "repository": "https://github.com/DefinitelyTyped/DefinitelyTyped",
    "license": "MIT"
  },
  {
    "name": "@types/react-dom@18.0.9",
    "repository": "https://github.com/DefinitelyTyped/DefinitelyTyped",
    "license": "MIT"
  },
  {
    "name": "@types/react-router-dom@5.3.3",
    "repository": "https://github.com/DefinitelyTyped/DefinitelyTyped",
    "license": "MIT"
  },
  {
    "name": "@types/react-timeago@4.1.3",
    "repository": "https://github.com/DefinitelyTyped/DefinitelyTyped",
    "license": "MIT"
  },
  {
    "name": "@types/react@18.0.26",
    "repository": "https://github.com/DefinitelyTyped/DefinitelyTyped",
    "license": "MIT"
  },
  {
    "name": "framer-motion@7.6.19",
    "repository": "https://github.com/framer/motion",
    "license": "MIT"
  },
  {
    "name": "lodash@4.17.21",
    "repository": "https://github.com/lodash/lodash",
    "license": "MIT"
  },
  {
    "name": "parcel@2.8.0",
    "repository": "https://github.com/parcel-bundler/parcel",
    "license": "MIT"
  },
  {
    "name": "process@0.11.10",
    "repository": "https://github.com/shtylman/node-process",
    "license": "MIT"
  },
  {
    "name": "react-dom@18.2.0",
    "repository": "https://github.com/facebook/react",
    "license": "MIT"
  },
  {
    "name": "react-router-dom@6.4.4",
    "repository": "https://github.com/remix-run/react-router",
    "license": "MIT"
  },
  {
    "name": "react-timeago@7.1.0",
    "repository": "https://github.com/naman34/react-timeago",
    "license": "MIT"
  },
  {
    "name": "react@18.2.0",
    "repository": "https://github.com/facebook/react",
    "license": "MIT"
  },
  {
    "name": "recoil@0.7.6",
    "repository": "https://github.com/facebookexperimental/Recoil",
    "license": "MIT"
  },
  {
    "name": "swr@1.3.0",
    "repository": "https://github.com/vercel/swr",
    "license": "MIT"
  },
  {
    "name": "typescript@4.9.3",
    "repository": "https://github.com/Microsoft/TypeScript",
    "license": "Apache-2.0"
  }
];
