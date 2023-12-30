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
        ];

        # Libraries needed at runtime
        runtimeDeps = with pkgs; [
          xorg.libXcursor
          xorg.libxcb
          freetype
          xorg.libXrandr
          libGL
        ] ++ sharedDeps;



        # Manually simulate a vcpkg installation so that it can link the libraries
        # properly. Borrowed and adapted from: https://github.com/NixOS/nixpkgs/blob/69a35ff92dc404bf04083be2fad4f3643b2152c9/pkgs/applications/networking/remote/rustdesk/default.nix#L51
        vcpkg = pkgs.stdenv.mkDerivation {
          pname = "vcpkg";
          version = "1.0.0";

          unpackPhase =
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
              mkdir -p vcpkg/.vcpkg-root
              mkdir -p vcpkg/installed/${vcpkg_target}/lib
              mkdir -p vcpkg/installed/vcpkg/updates
              ln -s ${updates_vcpkg_file} vcpkg/installed/vcpkg/status
              mkdir -p vcpkg/installed/vcpkg/info
              touch vcpkg/installed/vcpkg/info/libyuv_1.0_${vcpkg_target}.list
              touch vcpkg/installed/vcpkg/info/libvpx_1.0_${vcpkg_target}.list
              ln -s ${pkgs.libvpx.out}/lib/* vcpkg/installed/${vcpkg_target}/lib/
              ln -s ${pkgs.libyuv.out}/lib/* vcpkg/installed/${vcpkg_target}/lib/
            '';

          installPhase = ''
            cp -r vcpkg $out
          '';
        };

        envVars = rec {
          RUST_BACKTRACE = 1;
          # MOLD_PATH = "${pkgs.mold.out}/bin/mold";
          # RUSTFLAGS = "-Clink-arg=-fuse-ld=${MOLD_PATH} -Clinker=clang";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          VCPKG_ROOT = "${vcpkg.out}";
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

        buildInputs = with pkgs;
          [
            libvpx
            libyuv
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

        inherit (craneLib.crateNameFromCargoToml { cargoToml = ./app/Cargo.toml; }) pname version;

        cargoArtifacts = craneLib.buildDepsOnly ({

          inherit pname version src buildInputs nativeBuildInputs stdenv;

          # This magic thing here is needed otherwise the build fails
          # https://github.com/ipetkov/crane/issues/411#issuecomment-1743441117
          # Might revisit later to see if anything else pops up
          installCargoArtifactsMode = "use-zstd";
        } // envVars);

        # The main application derivation
        legion-kb-rgb = craneLib.buildPackage
          ({
            inherit pname version src cargoArtifacts buildInputs nativeBuildInputs stdenv;

            doCheck = false;

            # cargoBuildCommand = "cargo build";

            postFixup = ''
              patchelf --add-rpath "${nixLib.makeLibraryPath runtimeDeps}" "$out/bin/${pname}"
            '';
          } // envVars);

        # Wrap the program for ease of use
        wrappedProgram = pkgs.symlinkJoin rec {
          name = "legion-kb-rgb";
          paths = [ legion-kb-rgb ];

          buildInputs = [ pkgs.makeWrapper ];

          postBuild = ''
            wrapProgram $out/bin/${name} \
              --prefix LD_LIBRARY_PATH : ${nixLib.makeLibraryPath runtimeDeps}
          '';
        };
      in
      {
        checks = {
          # inherit legion-kb-rgb;
        };

        packages.default = legion-kb-rgb;
        packages.wrapped = wrappedProgram;

        apps.default = flake-utils.lib.mkApp {
          drv = wrappedProgram;
        };

        devShells.default = legion-kb-rgb;
        devShells.rust =
          let
            deps = buildInputs ++ nativeBuildInputs ++ sharedDeps ++ runtimeDeps;
          in
          pkgs.mkShell {
            LD_LIBRARY_PATH = nixLib.makeLibraryPath deps;
            inherit (envVars) VCPKG_ROOT LIBCLANG_PATH;

            buildInputs = [ rust ] ++ deps;
          };
      });
}
