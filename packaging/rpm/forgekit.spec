# RPM spec file for ForgeKit
# This file defines dependencies for the .rpm package
# When users install forgekit.rpm, these dependencies will be automatically installed

Name:           forgekit
Version:        0.0.9
Release:        1%{?dist}
Summary:        Local-first media and PDF toolkit
License:        MIT
URL:            https://github.com/nedanwr/forgekit
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust
Requires:       qpdf
Requires:       ghostscript
Requires:       tesseract
Requires:       ffmpeg
Requires:       libvips
Requires:       perl-Image-ExifTool
Requires:       python3
Requires:       python3-pip

%description
ForgeKit is a fast, privacy-focused toolkit for everyday media and PDF tasks.
It provides a unified CLI for operations like PDF merging, image conversion,
audio normalization, and video transcoding.

All operations run locally - no data leaves your device.

%prep
%setup -q

%build
cargo build --release

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 target/release/forgekit %{buildroot}%{_bindir}/forgekit

%post
# Automatically install ocrmypdf Python package after installation
# Note: Python3 is a required dependency, so it will be installed automatically
# by dnf/yum if not already present. This script uses the existing Python installation.
if command -v python3 >/dev/null 2>&1 && command -v pip3 >/dev/null 2>&1; then
    pip3 install --quiet --user ocrmypdf 2>/dev/null || pip3 install --quiet ocrmypdf 2>/dev/null || true
fi

%files
%{_bindir}/forgekit

%changelog
* Fri Jan 2026 ForgeKit Contributors <forgekit@example.com> - 0.0.9-1
- Added GitHub Actions release workflow
- Added Dockerfile for containerized distribution
- Added Makefile for common tasks
- Updated Cargo.toml with crates.io publishing fields

* Wed Dec 2025 ForgeKit Contributors <forgekit@example.com> - 0.0.3-1
- Initial RPM package

