#!/bin/sh
# Print the Homebrew formula for release TAG (v1.2.3) from the .sha256 files in DIR
# (as uploaded by taiki-e/upload-rust-binary-action: tempa-TAG-TARGET.sha256).
# Fails, printing nothing, unless every checksum is present and well formed.
set -eu
TAG=$1
DIR=$2
URL="https://github.com/anggaralabshq/tempa/releases/download/$TAG"
sum() {
  s=$(cut -d' ' -f1 "$DIR/tempa-$TAG-$1.sha256")
  echo "$s" | grep -Eqx '[0-9a-f]{64}' || { echo "bad checksum for $1: '$s'" >&2; exit 1; }
  echo "$s"
}
MAC_ARM=$(sum aarch64-apple-darwin)
MAC_X86=$(sum x86_64-apple-darwin)
LINUX_ARM=$(sum aarch64-unknown-linux-musl)
LINUX_X86=$(sum x86_64-unknown-linux-musl)
cat <<RUBY
class Tempa < Formula
  desc "One contract (AGENTS.md + .tempa/) kept in sync with every coding agent"
  homepage "https://github.com/anggaralabshq/tempa"
  version "${TAG#v}"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    on_arm do
      url "$URL/tempa-$TAG-aarch64-apple-darwin.tar.gz"
      sha256 "$MAC_ARM"
    end
    on_intel do
      url "$URL/tempa-$TAG-x86_64-apple-darwin.tar.gz"
      sha256 "$MAC_X86"
    end
  end

  on_linux do
    on_arm do
      url "$URL/tempa-$TAG-aarch64-unknown-linux-musl.tar.gz"
      sha256 "$LINUX_ARM"
    end
    on_intel do
      url "$URL/tempa-$TAG-x86_64-unknown-linux-musl.tar.gz"
      sha256 "$LINUX_X86"
    end
  end

  def install
    bin.install "tempa"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/tempa --version")
  end
end
RUBY
