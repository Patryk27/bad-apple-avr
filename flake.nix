{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-20.09";
    };
  };

  outputs = { self, nixpkgs }: {
    devShell = {
      x86_64-linux =
        let
          pkgs = (import nixpkgs) {
            system = "x86_64-linux";
          };

        in
        pkgs.mkShell {
          buildInputs = with pkgs; [
            avrdude
            pkgs.pkgsCross.avr.buildPackages.gcc
          ];
        };
    };
  };
}
