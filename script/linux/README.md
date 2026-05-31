# WarpCodexOss Linux / Ubuntu builds

WarpCodexOss is the OSS Linux package with the local Codex OAuth bridge enabled
by default. It does not bundle, read, copy, or parse Codex OAuth tokens. The
target machine must install Codex CLI separately and authenticate through the
official Codex login flow.

Supported Ubuntu targets for this local fork:

- Ubuntu 24.04 x64 / arm64
- Ubuntu 26.04 x64 / arm64

## Build prerequisites

Run these on the Ubuntu host or container:

```bash
./script/linux/install_build_deps
./script/linux/install_linuxdeploy
```

Codex CLI is not required to build the package. It is required on the machine
that runs WarpCodexOss. To validate runtime auth, run:

```bash
./script/linux/check_codex
```

`check_codex` only checks `codex --version` and `codex login status`; it never
reads `~/.codex/auth.json`.

## Check-only build

```bash
./script/linux/bundle_ubuntu_codex --ubuntu 24.04 --check-only
./script/linux/bundle_ubuntu_codex --ubuntu 26.04 --check-only
```

## Package build

```bash
./script/linux/bundle_ubuntu_codex --ubuntu 24.04 --packages deb,appimage
./script/linux/bundle_ubuntu_codex --ubuntu 26.04 --packages deb,appimage
```

## Docker build from macOS/Linux

If Docker is running, this wrapper builds inside the matching Ubuntu image:

```bash
./script/linux/docker_build_ubuntu_codex --ubuntu 24.04 --arch x86_64
./script/linux/docker_build_ubuntu_codex --ubuntu 26.04 --arch x86_64
```

On Apple Silicon, `--arch x86_64` uses Docker emulation and can be much slower.
Use `--arch aarch64` for native ARM64 packages.

Outputs are written under:

```text
target/<profile>/bundle/linux/
```

Expected local Codex artifacts:

- `warp-terminal-codex-oss_<version>_amd64.deb` or `arm64.deb`
- `WarpCodexOss-x86_64.AppImage` or `WarpCodexOss-aarch64.AppImage`

## Runtime expectations

- Installed desktop app name: `WarpCodexOss`
- Installed binary inside the package: `warp-codex-oss`
- Debian package name: `warp-terminal-codex-oss`
- The package does not auto-configure the official Warp apt repository, so a
  local WarpCodexOss install will not be silently replaced by upstream Warp.
- The runtime still uses `codex` from `PATH`, or `WARP_LOCAL_CODEX_BIN` if set.
- Set `WARP_LOCAL_CODEX_AI=0` only when you explicitly want to fall back to Warp
  AI credits.
