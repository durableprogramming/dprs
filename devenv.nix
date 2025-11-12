{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/basics/
  env.GREET = "devenv";

  # https://devenv.sh/packages/
  packages = [ pkgs.git pkgs.xorg.libxcb pkgs.bashInteractive pkgs.cmake pkgs.xorg.libX11.dev pkgs.cargo-deb pkgs.zig ];

  # https://devenv.sh/languages/
  languages.rust.enable = true;
  languages.rust.channel= "stable";
  languages.rust.components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer"];
  languages.rust.targets = [ "x86_64-unknown-linux-musl" ];


  # https://devenv.sh/scripts/
  scripts.hello.exec = ''
    echo hello from $GREET
  '';

  scripts.cargo-build-release.exec = ''
    cargo zigbuild --release --target x86_64-unknown-linux-musl "$@"
  '';

  enterShell = ''
    hello
    git --version
  '';

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep --color=auto "${pkgs.git.version}"
  '';

  # https://devenv.sh/pre-commit-hooks/
  # pre-commit.hooks.shellcheck.enable = true;

  # See full reference at https://devenv.sh/reference/options/
}

