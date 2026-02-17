{ lib
, fetchFromGitHub
, stdenv
}:

let
  version = "__VERSION__";
  pname = "elevenlabs-cli";

  sources = {
    x86_64-linux = {
      url = "https://github.com/hongkongkiwi/elevenlabs-cli/releases/download/v${version}/elevenlabs-cli-v${version}-x86_64-unknown-linux-musl.tar.gz";
      sha256 = "__SHA256_X86_64__";
    };
    aarch64-linux = {
      url = "https://github.com/hongkongkiwi/elevenlabs-cli/releases/download/v${version}/elevenlabs-cli-v${version}-aarch64-unknown-linux-musl.tar.gz";
      sha256 = "__SHA256_AARCH64__";
    };
    x86_64-darwin = {
      url = "https://github.com/hongkongkiwi/elevenlabs-cli/releases/download/v${version}/elevenlabs-cli-v${version}-x86_64-apple-darwin.tar.gz";
      sha256 = "__SHA256_X86_64_DARWIN__";
    };
    aarch64-darwin = {
      url = "https://github.com/hongkongkiwi/elevenlabs-cli/releases/download/v${version}/elevenlabs-cli-v${version}-aarch64-apple-darwin.tar.gz";
      sha256 = "__SHA256_AARCH64_DARWIN__";
    };
  };

  sourceInfo = sources.${stdenv.hostPlatform.system} or (throw "Unsupported system: ${stdenv.hostPlatform.system}");

in stdenv.mkDerivation {
  inherit pname version;

  src = fetchFromGitHub {
    owner = "hongkongkiwi";
    repo = "elevenlabs-cli";
    rev = "v${version}";
    sha256 = lib.fakeSha256;
  };

  # Use pre-built binary
  dontBuild = true;
  dontConfigure = true;

  installPhase = ''
    mkdir -p $out/bin
    cp elevenlabs-cli $out/bin/
    chmod +x $out/bin/elevenlabs-cli
  '';

  meta = with lib; {
    description = "Unofficial CLI for ElevenLabs text-to-speech API";
    homepage = "https://github.com/hongkongkiwi/elevenlabs-cli";
    license = licenses.mit;
    platforms = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    maintainers = [ ];
    mainProgram = "elevenlabs-cli";
  };
}