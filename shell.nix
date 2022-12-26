let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  # Pin to stable from https://status.nixos.org/
  nixpkgs = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/dac57a4eccf1442e8bf4030df6fcbb55883cb682.tar.gz") { overlays = [ moz_overlay ]; };
in
  with nixpkgs;
  stdenv.mkDerivation {
    name = "moz_overlay_shell";
    buildInputs = with nixpkgs; [
      ((rustChannelOf{ channel = "1.66.0"; }).rust.override {
        extensions = ["rust-src"];
      })
      cargo-watch
    ];
  }
