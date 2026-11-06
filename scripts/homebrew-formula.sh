#!/bin/sh
# Print the Homebrew formula for release TAG (v1.2.3) from the .sha256 files in DIR
# (as uploaded by the release workflow: tempa-TAG-TARGET.tar.gz.sha256).
set -eu
TAG=$1
DIR=$2
URL="https://github.com/anggaralabshq/tempa/releases/download/$TAG"
sum() { cut -d' ' -f1 "$DIR/tempa-$TAG-$1.tar.gz.sha256"; }
cat <<RUBY
class Tempa < Formula
  desc "One contract (AGENTS.md + .tempa/) kept in sync with every coding agent"
  homepage "https://github.com/anggaralabshq/tempa"
  version "${TAG#v}"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    on_arm do
      url "$URL/tempa-$TAG-aarch64-apple-darwin.tar.gz"
      sha256 "$(sum aarch64-apple-darwin)"
    end
    on_intel do
      url "$URL/tempa-$TAG-x86_64-apple-darwin.tar.gz"
      sha256 "$(sum x86_64-apple-darwin)"
    end
  end

  on_linux do
    on_arm do
      url "$URL/tempa-$TAG-aarch64-unknown-linux-musl.tar.gz"
      sha256 "$(sum aarch64-unknown-linux-musl)"
    end
    on_intel do
      url "$URL/tempa-$TAG-x86_64-unknown-linux-musl.tar.gz"
      sha256 "$(sum x86_64-unknown-linux-musl)"
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
