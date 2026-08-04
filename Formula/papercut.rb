class Papercut < Formula
  desc "A tiny CLI that gives AI agents a complaint box."
  homepage "https://github.com/dowhilelabs/papercut"
  version "0.1.5"
  url "https://github.com/dowhilelabs/papercut/releases/download/v0.1.5/papercut-aarch64-apple-darwin.tar.gz"
  # SHA-256 of papercut-aarch64-apple-darwin.tar.gz (v0.1.5 release).
  sha256 "eb7ee26df5e0d01ff77178b268d1d2e80f54210623e1c8169639d23081ea35c8"

  depends_on :macos

  def install
    bin.install "papercut"
  end

  test do
    # `schema` prints the machine contract as JSON; verify the CLI runs.
    assert_match '"ok":true', shell_output("#{bin}/papercut schema")
  end
end
