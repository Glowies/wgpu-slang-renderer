{
  pkgs ? import <nixpkgs> { },
}:
let
  libPath = with pkgs; lib.makeLibraryPath [
    libGL
    libxkbcommon
    wayland
  ];
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
  buildInputs = with pkgs; [
    openssl
    clang
    llvmPackages.bintools
    rustup
  ];

  LD_LIBRARY_PATH = libPath;

  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
  '';
}
