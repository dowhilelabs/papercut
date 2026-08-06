class Papercut < Formula
  desc "A tiny CLI that gives AI agents a complaint box."
  homepage "https://github.com/dowhilelabs/papercut"
  version "0.1.8"
  url "https://github.com/dowhilelabs/papercut/releases/download/v0.1.8/papercut-aarch64-apple-darwin.tar.gz"
  # SHA-256 of papercut-aarch64-apple-darwin.tar.gz (v0.1.8 release).
  sha256 "36face05a4d0f6f74624c6476ed4a2b09c196574ea211c5cc57feba78cc816ca"

  depends_on :macos

  def install
    bin.install "papercut"
  end

  test do
    # `schema` prints the machine contract as JSON; verify the CLI runs.
    assert_match '"ok":true', shell_output("#{bin}/papercut schema")
  end
end
