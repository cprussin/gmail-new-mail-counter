{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    mkCli.url = "github:cprussin/mkCli";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    mkCli,
    ...
  }: let
    cargo-with-overlay = final: _: {
      cargo-with = plugins:
        final.symlinkJoin {
          name = "cargo-with-plugins";
          paths = [final.cargo];
          buildInputs = [final.makeWrapper];
          postBuild = ''
            wrapProgram $out/bin/cargo \
              --prefix PATH : ${final.lib.makeBinPath ([final.cargo] ++ plugins)}
          '';
        };
    };

    cli-overlay = final: _: {
      cli = final.lib.mkCli "cli" {
        _noAll = true;

        run = "${final.cargo-with [final.cargo-watch]}/bin/cargo run";

        test = {
          rust = {
            audit = "${final.cargo-with [final.cargo-audit]}/bin/cargo audit";
            check = "${final.cargo}/bin/cargo check";
            format = "${final.cargo-with [final.rustfmt]}/bin/cargo fmt --check";
            lint = "${final.cargo-with [final.clippy]}/bin/cargo clippy -- --deny warnings";
            unit = "${final.cargo}/bin/cargo test";
            version-check = "${final.cargo-with [final.cargo-outdated]}/bin/cargo outdated";
          };
          nix = {
            flake = "${final.nix}/bin/nix flake check";
            lint = "${final.statix}/bin/statix check .";
            dead-code = "${final.deadnix}/bin/deadnix .";
            format = "${final.alejandra}/bin/alejandra --check .";
          };
        };

        fix = {
          rust = {
            format = "${final.cargo-with [final.rustfmt]}/bin/cargo fmt";
            lint = "${final.cargo-with [final.clippy]}/bin/cargo clippy --fix";
          };
          nix = {
            lint = "${final.statix}/bin/statix fix .";
            dead-code = "${final.deadnix}/bin/deadnix -e .";
            format = "${final.alejandra}/bin/alejandra .";
          };
        };
      };
    };

    gmail-new-mail-counter-overlay = final: _: {
      gmail-new-mail-counter = final.rustPlatform.buildRustPackage {
        name = "gmail-new-mail-counter";
        version = "0.1.0";
        src = final.lib.cleanSource ./.;
        cargoLock = {
          lockFile = ./Cargo.lock;
          allowBuiltinFetchGit = true;
        };
      };
    };
  in
    (
      flake-utils.lib.eachDefaultSystem
      (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [cargo-with-overlay mkCli.overlays.default cli-overlay gmail-new-mail-counter-overlay];
            config = {};
          };
        in {
          apps.default = {
            type = "app";
            program = "${pkgs.gmail-new-mail-counter}/bin/gmail_new_mail_counter";
          };

          packages.default = pkgs.gmail-new-mail-counter;

          devShells.default = pkgs.mkShell {
            buildInputs = [
              (pkgs.cargo-with [pkgs.rustfmt pkgs.clippy pkgs.cargo-outdated])
              pkgs.cli
              pkgs.git
              pkgs.openssl
              pkgs.pkg-config
              pkgs.rust-analyzer
              pkgs.rustc
            ];
          };
        }
      )
    )
    // {
      overlays.default = gmail-new-mail-counter-overlay;
    };
}
