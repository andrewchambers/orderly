{stdenv, rustPlatform, fetchFromGitHub}:
rustPlatform.buildRustPackage rec {
  name = "orderly-${version}";
  
  version = "0.2.0";

  src = fetchFromGitHub {
    owner = "andrewchambers";
    repo = "orderly";
    rev = "v${version}";
    sha256 = "0b7xk75lv0r2q7k1xrhgcn0lrzsbk9ps7h3imm32qqmq7z2dpfz1";
  };

  cargoSha256 = "17wxa3i6zza0kgh0gnldyrqm5rz38ydsdvga8dhdrdkw923b02i3";

  postInstall = ''
    mkdir -p $out/share/man/man1
    cat ./man/generated/orderly.1 | gzip > $out/share/man/man1/orderly.1.gz
  '';

  meta = with stdenv.lib; {
    description = "Ordered process (re)start, shutdown, and supervision.";
    homepage = https://github.com/andrewchambers/orderly;
  };
}