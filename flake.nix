{
  description = "Rocket Blog & Work Time Tracker";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
        
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];

        buildInputs = with pkgs; [
          openssl
        ];

        app = rustPlatform.buildRustPackage {
          pname = "rocket-blog";
          version = "0.1.0";
          src = ./.;
          cargoLock = { lockFile = ./Cargo.lock; };
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
          doCheck = false;
          OPENSSL_NO_VENDOR = 1;
          
          postInstall = ''
            mkdir -p $out/share/rocket-blog
            cp -r templates $out/share/rocket-blog/
            cp -r static $out/share/rocket-blog/
          '';
        };

      in
      {
        packages = {
          default = app;
          rocket-blog = app;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = nativeBuildInputs ++ [ rustToolchain ];
          inherit buildInputs;
          packages = with pkgs; [ sea-orm-cli cargo-watch just ];
          shellHook = ''
            echo "🦀 Rust Dev Environment Loaded"
            echo "Rust version: $(rustc --version)"
          '';
        };
      }) // {
      nixosModules.default = import ./nix/module.nix { inherit self; };
      nixosConfigurations.rocket-container = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          ({ config, pkgs, ... }: {
            boot.isContainer = true;
            imports = [ self.nixosModules.default ];
            system.stateVersion = "23.11";
            services.rocket-blog = {
              enable = true;
              domain = "blog.localhost";
              manageDatabase = true;
            };
          })
        ];
      };
    };
}
