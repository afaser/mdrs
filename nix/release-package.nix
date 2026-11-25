{
  pkgs,
  runtime,
}:
{ version, src }:
pkgs.stdenv.mkDerivation {
  pname = "mdrs";
  inherit version src;
  sourceRoot = ".";

  nativeBuildInputs = [
    pkgs.autoPatchelfHook
    pkgs.patchelf
  ];
  buildInputs = [
    pkgs.stdenv.cc.cc.lib
  ];
  runtimeDependencies = runtime;

  dontConfigure = true;
  dontBuild = true;

  installPhase = ''
    runHook preInstall
    mkdir -p $out/bin
    install -m755 mdrs $out/bin/mdrs
    runHook postInstall
  '';

  doInstallCheck = true;
  installCheckPhase = ''
    test -x $out/bin/mdrs
  '';

  meta = {
    description = "Markdown viewer";
    mainProgram = "mdrs";
    platforms = ["x86_64-linux"];
  };
}
