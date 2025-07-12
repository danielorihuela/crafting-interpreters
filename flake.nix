{
  description = "Nix flake for Crafting Interpreters implementations";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    nixpkgs-dart.url =
      "github:nixos/nixpkgs?ref=e040aab15638aaf8d0786894851a2b1ca09a7baf";

    flake-utils.url = "github:numtide/flake-utils";

  };

  outputs = { self, nixpkgs, nixpkgs-dart, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        dartPkgs = import nixpkgs-dart { inherit system; };
      in {
        devShells = {
          go = pkgs.mkShell {
            packages = [ pkgs.go ];
            shellHook = ''
              export GOPATH=$HOME/go
              export GOBIN=$GOPATH/bin
              export PATH=$GOBIN:$PATH

              cd lox-tw
            '';
          };
        };

        packages.test = pkgs.writeShellScriptBin "run-loxtw-tests" ''
          git=${pkgs.git}/bin/git
          go=${pkgs.go}/bin/go
          dart=${dartPkgs.dart}/bin/dart

          $git submodule init
          $git submodule update
          (cd lox-tw; $go build; $go test -v ./...)
          (cd craftinginterpreters/tool; $dart pub get > /dev/null)

          cd craftinginterpreters
          CHAPTER=4 $dart tool/bin/test.dart chap04_scanning --interpreter ../lox-tw/lox-tw
          CHAPTER=6 $dart tool/bin/test.dart chap06_parsing --interpreter ../lox-tw/lox-tw
          CHAPTER=7 $dart tool/bin/test.dart chap07_evaluating --interpreter ../lox-tw/lox-tw
          cd ..

          (cd lox-tw; $go clean)
          (cd craftinginterpreters/tool; $dart pub cache clean -f)
        '';
      });
}
