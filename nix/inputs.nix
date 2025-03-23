{ pkgs, ... }:

let
  runtimeInputs = with pkgs; [
    libxkbcommon cargo-generate
  ];

  buildInputs = with pkgs; [
    udev openssl_3 espup 
  ] ++ runtimeInputs;
in
{
  inherit buildInputs runtimeInputs;
}