{
  description = "Build env";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      # inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        # flake-utils.follows = "flake-utils";
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

        rustVersion = "1.84.0";

        rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
          extensions = [
            "rust-src" # rust-analyzer
          ];
        };

        nixLib = nixpkgs.lib;
        craneLib = (crane.mkLib pkgs).overrideToolchain rust;

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

          # Tray icon
          pango
          gtk3
          gdk-pixbuf
          xdotool
        ];

        # Libraries needed at runtime
        runtimeDeps = with pkgs; [
          xorg.libXcursor
          xorg.libxcb
          freetype
          xorg.libXrandr
          libGL
          wayland
          libxkbcommon

          # Tray icon
          libayatana-appindicator
        ] ++ sharedDeps;

        envVars = rec {
          RUST_BACKTRACE = 1;
          # MOLD_PATH = "${pkgs.mold.out}/bin/mold";
          # RUSTFLAGS = "-Clink-arg=-fuse-ld=${MOLD_PATH} -Clinker=clang";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };

        # Allow a few more files to be included in the build workspace
        workspaceSrc = ./.;
        workspaceSrcString = builtins.toString workspaceSrc;

        resFileFilter = path: _type: builtins.match "${workspaceSrcString}/app/res/.*" path != null;
        workspaceFilter = path: type:
          (resFileFilter path type) || (craneLib.filterCargoSources path type);


        src = nixLib.cleanSourceWith
          {
            src = workspaceSrc;
            filter = workspaceFilter;
          };

        # https://github.com/NixOS/nixpkgs/blob/nixos-unstable/pkgs/by-name/ru/rustdesk/package.nix
        buildInputs = with pkgs;
          [
            libvpx
            libyuv
            libaom
          ]
          ++ sharedDeps
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ ];

        nativeBuildInputs = with pkgs;
          [
            pkg-config
            cmake
            clang
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ ];

        stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;

        # Forgo using VCPKG hacks on local builds because pain
        cargoExtraArgs = nixLib.optionals stdenv.isLinux ''--locked --features "scrap/linux-pkg-config"'';

        inherit (craneLib.crateNameFromCargoToml { cargoToml = ./app/Cargo.toml; }) pname version;

        cargoArtifacts = craneLib.buildDepsOnly ({

          inherit pname version src buildInputs nativeBuildInputs stdenv cargoExtraArgs;

          # This magic thing here is needed otherwise the build fails
          # https://github.com/ipetkov/crane/issues/411#issuecomment-1743441117
          # Might revisit later to see if anything else pops up
          installCargoArtifactsMode = "use-zstd";
        } // envVars);

        # The main application derivation
        legion-kb-rgb = craneLib.buildPackage
          ({
            inherit pname version src cargoArtifacts buildInputs nativeBuildInputs stdenv cargoExtraArgs;

            doCheck = false;

            # cargoBuildCommand = "cargo build";

            postFixup = ''
              patchelf --add-rpath "${nixLib.makeLibraryPath runtimeDeps}" "$out/bin/${pname}"
            '';
          } // envVars);
      in
      {
        checks = {
          # inherit legion-kb-rgb;
        };

        packages.default = legion-kb-rgb;

        apps.default = flake-utils.lib.mkApp {
          drv = legion-kb-rgb;
        };

        devShells.default = legion-kb-rgb;
        devShells.rust =
          let
            deps = buildInputs ++ nativeBuildInputs ++ sharedDeps ++ runtimeDeps;
          in
          pkgs.mkShell {
            LD_LIBRARY_PATH = nixLib.makeLibraryPath deps;
            inherit (envVars) LIBCLANG_PATH;

            buildInputs = [ rust ] ++ deps;
          };
      });
}
