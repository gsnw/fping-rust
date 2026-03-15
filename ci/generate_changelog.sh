#!/usr/bin/env bash
set -e

LAST_TAG=$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || echo "")

if [ -z "$LAST_TAG" ]; then
  echo "# Changelog" > CHANGELOG.md
  git log --pretty=format:"- %s" >> CHANGELOG.md
else
  echo "# Changelog" > CHANGELOG.md
  git log ${LAST_TAG}..HEAD --pretty=format:"- %s" >> CHANGELOG.md
fi