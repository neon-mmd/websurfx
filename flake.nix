{
  # Websurfx NixOS flake
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    naersk,
    nixpkgs,
    self,
    utils,
  }:
  # We do this for all systems - namely x86_64-linux, aarch64-linux,
  # x86_64-darwin and aarch64-darwin
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      naersk-lib = pkgs.callPackage naersk {};
    in rec {
      # Build via "nix build .#default"
      packages.default = naersk-lib.buildPackage {
        # The build dependencies
        buildInputs = with pkgs; [pkg-config openssl];
        src = ./.;
      };

      # Enter devshell with all the tools via "nix develop"
      # or "nix-shell"
      devShells.default = with pkgs;
        mkShell {
          buildInputs = [
            actionlint
            cargo
            haskellPackages.hadolint
            nodePackages_latest.cspell
            nodePackages_latest.eslint
            nodePackages_latest.markdownlint-cli2
            nodePackages_latest.stylelint
            redis
            rustPackages.clippy
            rustc
            yamllint
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };

      # Build via "nix build .#websurfx", which is basically just
      # calls the build function
      packages.websurfx = packages.default;
    });
}
