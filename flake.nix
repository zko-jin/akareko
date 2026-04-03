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
        ];
      in
      {
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            clang
            clang-tools
            rust-analyzer
            boost
            boost-build
            # diesel-cli
          ];

          buildInputs = with pkgs; [
            openssl
            rust-bin.nightly.latest.default
            sqlite
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
            pkg-config
            libxkbcommon
            makeWrapper
            libGL
            wayland
            xdotool
          ];

          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;

          env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath dlopenLibraries}";
        };
      }
    );
}
