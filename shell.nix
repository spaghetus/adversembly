{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    openssl
    pkg-config
    wayland
    xorg.libX11
    libGL
    libxkbcommon
    wayland
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    alsa-lib
  ];
  LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
    libGL
    libxkbcommon
    wayland
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    alsa-lib
  ];
}
