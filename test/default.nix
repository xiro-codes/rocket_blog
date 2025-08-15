{pkgs ? import <nixos> {}}:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [nodejs cypress];

  shellHook = ''
    export CYPRESS_INSTALL_BINARY=0
    export CYPRESS_RUN_BINARY=${pkgs.cypress}/bin/Cypress
  '';
}
