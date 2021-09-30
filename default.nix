let
	pkgs = import <nixpkgs> {};
in
	with pkgs;
    stdenv.mkDerivation {
        name = "transaction_processor_build";
        buildInputs = [
    		pkg-config
    		cmake
    		clang
    		llvmPackages.libclang
    		rust-analyzer
        ];
        shellHook = ''
        	export LIBCLANG_PATH="${pkgs.llvmPackages.libclang}/lib";
        '';
    }
