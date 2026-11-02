class Daypress < Formula
  desc "Self-Hosted Rust CMS with 1-Click WP Migration, Daystudio Visual Builder & MCP Server"
  homepage "https://daypress.dev"
  version "{{VERSION}}"
  license "AGPL-3.0-or-later"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Daypress/daypress/releases/download/v#{version}/daypress-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "{{SHA256_DARWIN_ARM64}}"
    else
      url "https://github.com/Daypress/daypress/releases/download/v#{version}/daypress-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "{{SHA256_DARWIN_AMD64}}"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/Daypress/daypress/releases/download/v#{version}/daypress-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "{{SHA256_LINUX_AMD64}}"
    end
  end

  def install
    bin.install "daypress"
    (var/"lib/daypress").mkpath
  end

  service do
    run [opt_bin/"daypress", "--db", var/"lib/daypress/daypress.sqlite"]
    keep_alive true
    working_dir var/"lib/daypress"
    log_path var/"log/daypress.log"
    error_log_path var/"log/daypress.err.log"
    environment_variables PORT: "8090", DAYPRESS_NO_BROWSER: "1"
  end

  test do
    assert_match "Daypress", shell_output("#{bin}/daypress --help")
  end
end
