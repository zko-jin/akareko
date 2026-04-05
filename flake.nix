{
  description = "Akareko Development Shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        dlopenLibraries = with pkgs; [
          libGL
          libxkbcommon
          vulkan-loader
          libappindicator-gtk3
          libayatana-appindicator
          wayland
          boost
        ];

        libPaths = pkgs.lib.makeLibraryPath dlopenLibraries;
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            openssl
            # sqlite
            glib
            freetype
            fontconfig
            cairo
            pango
            gtk3
            libappindicator-gtk3
            libayatana-appindicator
          ];

          nativeBuildInputs = with pkgs; [
            rust-bin.nightly.latest.default
            pkg-config
            libxkbcommon
            makeWrapper
            libGL
            wayland
            xdotool

            clang
            clang-tools
            boost
            rust-analyzer
            boost-build
            dioxus-cli
            lld
          ];

          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;

          env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${libPaths}";
          env.LD_LIBRARY_PATH = libPaths;
        };
      }
    );
}
