{
  description = "Build env";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustVersion = "1.73.0";

        rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
          extensions = [
            "rust-src" # rust-analyzer
          ];
        };

        nixLib = nixpkgs.lib;

        # Libraries needed both at compile and runtime
        sharedDeps = with pkgs; [
          dbus
          xorg.libX11
          fontconfig
          udev
          glib
          gst_all_1.gstreamer
          gst_all_1.gst-plugins-base
          xorg.libXi
          libusb1
          expat
        ];

        # Libraries needed at runtime
        runtimeDeps = with pkgs; [
          xorg.libXcursor
          xorg.libxcb
          freetype
          xorg.libXrandr
          libGL

          mesa
          xorg.libX11
          xorg.libXxf86vm
          xorg.libXi

          wayland
          libxkbcommon
        ] ++ sharedDeps;
      in
      {
        checks = {
          # inherit legion-kb-rgb;
        };

        devShells.default = pkgs.mkShell {
          LD_LIBRARY_PATH = nixLib.makeLibraryPath runtimeDeps;

          packages = [ rust ];
        };
      });
}
