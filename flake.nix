{
  description = "mdrs markdown viewer";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
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
    in {
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
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtime;
      };
    };
}
