{
  description = "Akareko Development Shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        dlopenLibraries = with pkgs; [
          libxkbcommon
          vulkan-loader
          wayland
        ];
      in
      {
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            clang
            clang-tools
            rust-analyzer
            rustc
            rustfmt
            rustup

            boost
            boost-build
            diesel-cli
          ];

          buildInputs = with pkgs; [
            openssl
            sqlite
          ];

          nativeBuildInputs = with pkgs; [
          ];

          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;

          env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath dlopenLibraries}";
        };
      }
    );
}
