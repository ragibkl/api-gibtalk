#!/bin/sh
# Rewrites tests/media-paths.txt: every picture served under /media/. Run it
# after adding pictures. The served_media_paths_never_disappear test fails if
# a listed picture is renamed or removed, which would break boards, templates
# and share links that already use it.
set -e
cd "$(dirname "$0")/../media"
find . -type f ! -name "*.yaml" | sed 's#^\./##' | LC_ALL=C sort > ../tests/media-paths.txt
