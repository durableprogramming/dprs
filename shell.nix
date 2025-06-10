let
  pkgs = import <nixpkgs> {
    system = builtins.currentSystem;
    config = {
      allowUnfree = false;
      allowBroken = false;
      permittedInsecurePackages = [ ];
    };
  };

in
pkgs.mkShell {
  name = "project-dev-shell";

  buildInputs = with pkgs; [
    rustc
    cargo
    rust-analyzer
    clippy
    rustfmt

    git
    bashInteractive
    coreutils
    gnugrep
    gnused
    gawk
    findutils
    xorg.libxcb
  ];

  shellHook = ''
    echo "---------------------------------------------------------------------"
    echo "Welcome to your Nix development shell!"
    echo "Rust toolchain and development tools are now available."
    echo "Using system nixpkgs."
    echo "---------------------------------------------------------------------"
    export TMPDIR=~/.tmp
  '';
}
