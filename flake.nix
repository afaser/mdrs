{
  description = "mdrs markdown viewer";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      lib = pkgs.lib;
      version = (lib.importTOML ./Cargo.toml).package.version;
      runtime = with pkgs; [
        dbus
        vulkan-loader
        wayland
        libxkbcommon
        libx11
        libxcb
        libxcursor
        libxrandr
        libxi
        libxinerama
      ];
      mkReleasePackage = import ./nix/release-package.nix {
        inherit pkgs runtime;
      };
      release =
        if builtins.pathExists ./release.json
        then builtins.fromJSON (builtins.readFile ./release.json)
        else null;
      source = pkgs.rustPlatform.buildRustPackage {
        pname = "mdrs";
        inherit version;
        src = self;
        cargoLock.lockFile = ./Cargo.lock;
        nativeBuildInputs = [
          pkgs.autoPatchelfHook
          pkgs.patchelf
        ];
        buildInputs = [
          pkgs.stdenv.cc.cc.lib
        ];
        runtimeDependencies = runtime;
        doInstallCheck = true;
        installCheckPhase = ''
          test -x $out/bin/mdrs
        '';
        meta = {
          description = "Markdown viewer";
          mainProgram = "mdrs";
          platforms = ["x86_64-linux"];
        };
      };
      packages = rec {
        inherit source;
        default =
          if release == null
          then source
          else mkReleasePackage {
            inherit (release) version;
            src = pkgs.fetchurl {
              inherit (release) url;
              hash = release.hash;
            };
          };
        mdrs = default;
      };
    in {
      packages.${system} = packages;

      checks.${system} = {
        inherit (packages) source default;
      };

      apps.${system}.default = {
        type = "app";
        program = "${packages.default}/bin/mdrs";
        meta = {
          description = "Markdown viewer";
        };
      };

      lib.mkReleasePackage = mkReleasePackage;

      devShells.${system}.default = pkgs.mkShell {
        packages = runtime ++ (with pkgs; [
          cargo
          clippy
          direnv
          grim
          hyperfine
          rustc
          rustfmt
          sway
          vulkan-tools
          wtype
          xdotool
          xwayland
        ]);
        LD_LIBRARY_PATH = lib.makeLibraryPath runtime;
      };
    };
}
