# SPDX-License-Identifier: GPL-3.0-or-later
{ lib, rustPlatform }:
let
  # Content-addressed source: the installation media and installed systems
  # must produce the same derivation wherever this directory lives.
  source = builtins.path {
    path = ./.;
    name = "calamares-tatami-tool-src";
    filter =
      path: _:
      !(builtins.elem (baseNameOf path) [
        "target"
        "package.nix"
      ]);
  };
in
rustPlatform.buildRustPackage {
  pname = "calamares-tatami-tool";
  version = "0.1.0";
  src = source;
  cargoLock.lockFile = "${source}/Cargo.lock";
  meta = {
    description = "Session launcher and desktop commands for Tatami";
    license = lib.licenses.gpl3Plus;
    mainProgram = "tatami";
  };
}
