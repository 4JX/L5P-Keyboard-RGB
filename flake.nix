{
  description = "Cross platform software to control the RGB/lighting of the 4 zone keyboard included in the 2020, 2021, 2022, 2023 and 2024 lineup of the Lenovo Legion laptops";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-parts.url = "github:hercules-ci/flake-parts";

    systems = {
      url = "github:nix-systems/default-linux";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      nixpkgs,
      flake-parts,
      systems,
      crane,
      rust-overlay,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import systems;

      perSystem =
        {
          pkgs,
          system,
          lib,
          ...
        }:
        let
          rustVersion = "1.94.0";

          rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
            extensions = [
              "rust-src" # rust-analyzer
            ];
          };

          craneLib = (crane.mkLib pkgs).overrideToolchain rust;

          # Libraries needed both at compile and runtime
          sharedDeps = with pkgs; [
            dbus
            libx11
            fontconfig
            udev
            glib
            gst_all_1.gstreamer
            gst_all_1.gst-plugins-base
            libxi
            libusb1
            expat
            openssl

            # Tray icon
            pango
            gtk3
            gdk-pixbuf
            xdotool
          ];

          # Libraries needed at runtime
          runtimeDeps =
            with pkgs;
            [
              libxcursor
              libxcb
              freetype
              libxrandr
              libGL
              wayland
              libxkbcommon

              # Tray icon
              libayatana-appindicator
            ]
            ++ sharedDeps;

          buildEnvVars = {
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          };

          # Allow a few more files to be included in the build workspace
          workspaceSrc = ./.;
          workspaceSrcString = builtins.toString workspaceSrc;

          resFileFilter = path: _type: lib.hasPrefix "${workspaceSrcString}/app/res/" path;
          workspaceFilter = path: type: (resFileFilter path type) || (craneLib.filterCargoSources path type);

          src = lib.cleanSourceWith {
            src = workspaceSrc;
            filter = workspaceFilter;
          };

          # https://github.com/NixOS/nixpkgs/blob/nixos-unstable/pkgs/by-name/ru/rustdesk/package.nix
          buildInputs =
            with pkgs;
            [
              libvpx
              libyuv
              libaom
            ]
            ++ sharedDeps;

          nativeBuildInputs = with pkgs; [
            pkg-config
            cmake
            clang
          ];

          # Forgo using VCPKG hacks on local builds because pain
          cargoExtraArgs = ''--locked --features "scrap/linux-pkg-config"'';

          stdenv = p: (p.stdenvAdapters.useMoldLinker p.stdenv);

          inherit (craneLib.crateNameFromCargoToml { cargoToml = ./app/Cargo.toml; }) pname version;

          # Vendor dependencies to fix webm-sys compile issue
          # https://github.com/NixOS/nixpkgs/pull/475893
          # https://crane.dev/patching_dependency_sources.html
          cargoVendorDir = craneLib.vendorCargoDeps {
            inherit src;

            overrideVendorGitCheckout =
              ps: drv:
              let
                hasPackageNamed = name: lib.any (p: p.name == name) ps;
                isRustWebmRepo = lib.any (
                  p: lib.hasPrefix "git+https://github.com/rustdesk-org/rust-webm" p.source
                ) ps;
              in
              # Technically both webm and webm-sys come from the same repo/"set"
              if isRustWebmRepo && (hasPackageNamed "webm-sys" || hasPackageNamed "webm") then
                drv.overrideAttrs (old: {
                  postPatch = (old.postPatch or "") + ''
                    sed -e '1i #include <cstdint>' -i "src/sys/libwebm/mkvparser/mkvparser.cc"
                  '';
                })
              else
                drv;
          };

          commonArgs = {
            inherit
              pname
              version
              src
              buildInputs
              nativeBuildInputs
              cargoExtraArgs
              stdenv
              cargoVendorDir
              ;
          }
          // buildEnvVars;

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          # The main application derivation
          legion-kb-rgb = craneLib.buildPackage (
            commonArgs
            // {
              meta.mainProgram = pname;
              inherit cargoArtifacts;

              doCheck = false;

              postFixup = ''
                patchelf --add-rpath "${lib.makeLibraryPath runtimeDeps}" "$out/bin/${pname}"
              '';
            }
          );
        in
        {
          _module.args.pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };

          packages.default = legion-kb-rgb;

          apps.default.program = "${legion-kb-rgb}/bin/${pname}";

          devShells.default =
            let
              deps = buildInputs ++ nativeBuildInputs ++ runtimeDeps;
            in
            pkgs.mkShell {
              LD_LIBRARY_PATH = lib.makeLibraryPath deps;
              RUST_BACKTRACE = "1";
              inherit (buildEnvVars) LIBCLANG_PATH;

              buildInputs = [ rust ] ++ deps;
            };
        };
    };
}
