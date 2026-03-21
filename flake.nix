{
  description = "Websurfx NixOS flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    # We do this for all systems - namely x86_64-linux, aarch64-linux,
    # x86_64-darwin and aarch64-darwin
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      rec {
        # Build via "nix build"
        packages.default = pkgs.rustPlatform.buildRustPackage {
          name = "websurfx";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];

          # Copys and links files directly into the package
          postPatch = ''
            substituteInPlace src/handler.rs \
              --replace-fail "/etc/xdg" "$out/etc/xdg" \
              --replace-fail "/opt/websurfx" "$out/opt/websurfx"
          '';
          postInstall = ''
            mkdir -p $out/etc/xdg
            mkdir -p $out/opt/websurfx

            cp -r websurfx $out/etc/xdg/
            cp -r public $out/opt/websurfx/
          '';
        };

        # Enter devshell with all the tools via "nix develop"
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            actionlint
            cargo
            docker
            haskellPackages.hadolint
            nodejs
            nodePackages_latest.cspell
            eslint
            nodePackages_latest.markdownlint-cli2
            nodePackages_latest.stylelint
            redis
            rustPackages.clippy
            rust-analyzer
            cargo-watch
            rustc
            rustfmt
            yamllint
            openssl
            pkg-config
          ];
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          shellHook = ''
            export PATH="$PATH:$HOME/.cargo/bin"
            export NODE_PATH="$NODE_PATH:./node_modules"
          '';
        };

        # Build via "nix build .#websurfx"
        packages.websurfx = packages.default;
      }
    );
}
