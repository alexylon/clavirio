#!/usr/bin/env bash
# Builds a signed apt repository from the .deb files in a folder.
#
# Usage: apt-repo.sh <folder with .deb files> <output folder>
#
# The signing key must already be imported into the gpg keyring (GNUPGHOME).
# Only the given packages end up in the repository, so it always holds the
# latest release.
set -euo pipefail

debs="$1"
out="$2"
suite="stable"
component="main"
archs="amd64 arm64"

mkdir -p "${out}/pool/${component}"
cp "${debs}"/*.deb "${out}/pool/${component}/"
cd "${out}"

for arch in ${archs}; do
  dir="dists/${suite}/${component}/binary-${arch}"
  mkdir -p "${dir}"
  apt-ftparchive --arch "${arch}" packages "pool/${component}" > "${dir}/Packages"
  gzip -9kn "${dir}/Packages"
done

apt-ftparchive \
  -o APT::FTPArchive::Release::Origin=clavirio \
  -o APT::FTPArchive::Release::Label=clavirio \
  -o APT::FTPArchive::Release::Suite="${suite}" \
  -o APT::FTPArchive::Release::Codename="${suite}" \
  -o APT::FTPArchive::Release::Architectures="${archs}" \
  -o APT::FTPArchive::Release::Components="${component}" \
  release "dists/${suite}" > Release
mv Release "dists/${suite}/Release"

gpg --batch --yes --clearsign -o "dists/${suite}/InRelease" "dists/${suite}/Release"
gpg --batch --yes --armor --detach-sign -o "dists/${suite}/Release.gpg" "dists/${suite}/Release"

# The public key, in the binary format apt reads from /etc/apt/keyrings
gpg --batch --export > clavirio.gpg
