class Papercuts < Formula
  desc "A tiny CLI that gives AI agents a complaint box."
  homepage "https://github.com/treygoff24/papercuts"
  version "0.1.0"
  url "https://github.com/treygoff24/papercuts/releases/download/v0.1.0/papercuts-aarch64-apple-darwin.tar.gz"
  # Replace with the SHA-256 from the release's `papercuts-aarch64-apple-darwin.sha256`.
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"

  depends_on :macos

  def install
    bin.install "papercuts"
  end

  test do
    # `schema` prints the machine contract as JSON; verify the CLI runs.
    assert_match '"ok":true', shell_output("#{bin}/papercuts schema")
  end
end
