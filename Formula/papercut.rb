class Papercut < Formula
  desc "A tiny CLI that gives AI agents a complaint box."
  homepage "https://github.com/dowhilelabs/papercut"
  version "0.1.0"
  url "https://github.com/dowhilelabs/papercut/releases/download/v0.1.0/papercut-aarch64-apple-darwin.tar.gz"
  # SHA-256 of papercut-aarch64-apple-darwin.tar.gz (v0.1.0 release).
  sha256 "e6a5564ccd87113b8d28e6f2f9175256d79e286508ea3fb46c19318492c370b1"

  depends_on :macos

  def install
    bin.install "papercut"
  end

  test do
    # `schema` prints the machine contract as JSON; verify the CLI runs.
    assert_match '"ok":true', shell_output("#{bin}/papercut schema")
  end
end
