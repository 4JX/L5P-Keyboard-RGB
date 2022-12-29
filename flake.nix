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

        rustVersion = "1.65.0";

        rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
          extensions = [
            "rust-src" # rust-analyzer
          ];
        };

        nixLib = nixpkgs.lib;
        craneLib = (crane.mkLib pkgs).overrideToolchain rust;

        # TODO: Make this more specific
        protoFilter = path: _type: builtins.match ".*proto$" path != null;
        vpxHeaderFileFilter = path: _type: builtins.match ".*h$" path != null;
        resFileFilter = path: _type: builtins.match ".*/res/.*" path != null;
        workspaceFilter = path: type:
          (protoFilter path type) || (vpxHeaderFileFilter path type) || (resFileFilter path type) || (craneLib.filterCargoSources path type);

        my-crate = craneLib.buildPackage
          rec {
            src = nixLib.cleanSourceWith
              {
                src = ./.;
                filter = workspaceFilter;
              };

            buildInputs = with pkgs;
              [
                dbus
                xorg.libX11
                libusb
                pango
                libvpx
                libyuv
                xorg.libXinerama
                xorg.libXcursor
                xorg.libXfixes
              ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [

                # pkgs.libiconv
              ];

            nativeBuildInputs = with pkgs;
              [
                pkg-config
                git
                cmake
                clang
                gcc
                ninja
              ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [

                # pkgs.libiconv
              ];

            # Manually simulate a vcpkg installation so that it can link the libaries
            # properly. Borrowed from: https://github.com/NixOS/nixpkgs/blob/69a35ff92dc404bf04083be2fad4f3643b2152c9/pkgs/applications/networking/remote/rustdesk/default.nix#L51
            postUnpack =
              let
                vcpkg_target = "x64-linux";

                updates_vcpkg_file = pkgs.writeText "update_vcpkg_legion-kb-rgb"
                  ''
                    Package : libyuv
                    Architecture : ${vcpkg_target}
                    Version : 1.0
                    Status : is installed
                    Package : libvpx
                    Architecture : ${vcpkg_target}
                    Version : 1.0
                    Status : is installed
                  '';
              in
              ''
                export VCPKG_ROOT="$TMP/vcpkg"
                mkdir -p $VCPKG_ROOT/.vcpkg-root
                mkdir -p $VCPKG_ROOT/installed/${vcpkg_target}/lib
                mkdir -p $VCPKG_ROOT/installed/vcpkg/updates
                ln -s ${updates_vcpkg_file} $VCPKG_ROOT/installed/vcpkg/status
                mkdir -p $VCPKG_ROOT/installed/vcpkg/info
                touch $VCPKG_ROOT/installed/vcpkg/info/libyuv_1.0_${vcpkg_target}.list
                touch $VCPKG_ROOT/installed/vcpkg/info/libvpx_1.0_${vcpkg_target}.list
                ln -s ${pkgs.libvpx.out}/lib/* $VCPKG_ROOT/installed/${vcpkg_target}/lib/
                ln -s ${pkgs.libyuv.out}/lib/* $VCPKG_ROOT/installed/${vcpkg_target}/lib/
              '';

            RUST_BACKTRACE = 1;
            MOLD_PATH = "${pkgs.mold.out}/bin/mold";
            RUSTFLAGS = "-Clink-arg=-fuse-ld=${MOLD_PATH} -Clinker=clang";
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          };
      in
      {
        checks = {
          inherit my-crate;
        };

        packages.default = my-crate;

        apps.default = flake-utils.lib.mkApp {
          drv = my-crate;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;

          inherit (my-crate) buildInputs;

          # Extra inputs can be added here
          nativeBuildInputs = [
            rust
          ];
        };
      });
}
