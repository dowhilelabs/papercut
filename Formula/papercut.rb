class Papercut < Formula
  desc "A tiny CLI that gives AI agents a complaint box."
  homepage "https://github.com/dowhilelabs/papercut"
  version "0.1.4"
  url "https://github.com/dowhilelabs/papercut/releases/download/v0.1.4/papercut-aarch64-apple-darwin.tar.gz"
  # SHA-256 of papercut-aarch64-apple-darwin.tar.gz (v0.1.4 release).
  sha256 "7b9e50c29033a1c3892bc934e2e150013971773979001a6066c9260c98cce2ad"

  depends_on :macos

  def install
    bin.install "papercut"
  end

  test do
    # `schema` prints the machine contract as JSON; verify the CLI runs.
    assert_match '"ok":true', shell_output("#{bin}/papercut schema")
  end
end
