# Homebrew formula for ForgeKit
# This file defines dependencies for Homebrew installation
# When users install via `brew install forgekit`, these dependencies will be automatically installed

class Forgekit < Formula
  desc "Local-first media and PDF toolkit"
  homepage "https://github.com/nedanwar/forgekit"
  url "https://github.com/nedanwar/forgekit/releases/download/v0.0.3/forgekit-0.0.3.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"

  depends_on "rust" => :build
  depends_on "qpdf"
  depends_on "pdfcpu"
  depends_on "tesseract"
  depends_on "ffmpeg"
  depends_on "libvips"
  depends_on "exiftool"
  depends_on "python@3"

  def install
    # Build the workspace (both core and CLI crates)
    system "cargo", "build", "--release", "--workspace"
    
    # Install the binary
    bin.install "target/release/forgekit" => "forgekit"
  end

  post_install do
    # Automatically install ocrmypdf Python package
    # Python@3 is a required dependency, so it will be installed by Homebrew
    # if not already present. Homebrew automatically skips installing dependencies
    # that are already installed, so this works whether Python was just installed
    # or was already on the system.
    python3 = Formula["python@3"].opt_bin/"python3"
    system python3, "-m", "pip", "install", "--quiet", "--user", "ocrmypdf"
  end

  test do
    system "#{bin}/forgekit", "--version"
  end
end

