class Papercut < Formula
  desc "A tiny CLI that gives AI agents a complaint box."
  homepage "https://github.com/dowhilelabs/papercut"
  version "0.1.7"
  url "https://github.com/dowhilelabs/papercut/releases/download/v0.1.7/papercut-aarch64-apple-darwin.tar.gz"
  # SHA-256 of papercut-aarch64-apple-darwin.tar.gz (v0.1.7 release).
  sha256 "aab9488e298c4a345da1c6bd35c9e23fe59c919db1285033e44d0e34081c29ee"

  depends_on :macos

  def install
    bin.install "papercut"
  end

  test do
    # `schema` prints the machine contract as JSON; verify the CLI runs.
    assert_match '"ok":true', shell_output("#{bin}/papercut schema")
  end
end
