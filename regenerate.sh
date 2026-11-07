#!/bin/bash
set -euo pipefail

# Regenerate the Firecracker SDK models from the upstream OpenAPI spec.
#
# This is the same script the sync workflow runs, so a local run produces the
# same result as CI.
#
# Environment:
#   FC_TAG                     Firecracker release to generate from
#                              (default: the latest release)
#   OPENAPI_GENERATOR_VERSION  openapi-generator version (default: the version
#                              pinned in openapitools.json)
#   GITHUB_TOKEN               optional, authenticates GitHub API requests
#
# Hand-written code (src/lib.rs, src/machine.rs, src/client.rs, src/error.rs,
# Cargo.toml) is protected by .openapi-generator-ignore; edit it in place.

cd "$(dirname "${BASH_SOURCE[0]}")"

UPSTREAM_REPO="firecracker-microvm/firecracker"
SPEC_FILE="firecracker.yaml"

command_exists() {
	command -v "$1" >/dev/null 2>&1
}

# fetch URL [OUTPUT]: fail on HTTP errors instead of saving an error page.
fetch() {
	local auth=()
	if [ -n "${GITHUB_TOKEN:-}" ] && [[ "$1" == https://api.github.com/* ]]; then
		auth=(-H "Authorization: Bearer ${GITHUB_TOKEN}")
	fi
	if [ $# -gt 1 ]; then
		curl -fsSL "${auth[@]}" "$1" -o "$2"
	else
		curl -fsSL "${auth[@]}" "$1"
	fi
}

for cmd in curl java cargo; do
	if ! command_exists "$cmd"; then
		echo "ERROR: $cmd is required." >&2
		exit 1
	fi
done

if [ -z "${FC_TAG:-}" ]; then
	echo "==> Fetching latest Firecracker release tag..."
	FC_TAG=$(fetch "https://api.github.com/repos/${UPSTREAM_REPO}/releases/latest" |
		sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)
	if [ -z "$FC_TAG" ]; then
		echo "ERROR: Could not determine the latest release tag." >&2
		exit 1
	fi
fi
echo "    Firecracker release: ${FC_TAG}"

# The SDK is versioned and tagged like the Firecracker release it tracks.
SDK_VERSION="${FC_TAG#v}"
if ! [[ "$SDK_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
	echo "ERROR: Cannot derive an SDK version from tag ${FC_TAG}." >&2
	exit 1
fi

# The generator version is pinned so a new generator release can't change the
# output (or break the build) on its own. Bump it in openapitools.json.
if [ -z "${OPENAPI_GENERATOR_VERSION:-}" ]; then
	OPENAPI_GENERATOR_VERSION=$(sed -n 's/.*"version": *"\([^"]*\)".*/\1/p' openapitools.json | head -1)
fi
if [ -z "$OPENAPI_GENERATOR_VERSION" ]; then
	echo "ERROR: No openapi-generator version pinned in openapitools.json." >&2
	exit 1
fi
echo "    openapi-generator: v${OPENAPI_GENERATOR_VERSION}"

GENERATOR_JAR=".openapi-generator/openapi-generator-cli-${OPENAPI_GENERATOR_VERSION}.jar"
if [ ! -f "$GENERATOR_JAR" ]; then
	echo "==> Downloading openapi-generator v${OPENAPI_GENERATOR_VERSION}..."
	mkdir -p .openapi-generator
	fetch "https://repo1.maven.org/maven2/org/openapitools/openapi-generator-cli/${OPENAPI_GENERATOR_VERSION}/openapi-generator-cli-${OPENAPI_GENERATOR_VERSION}.jar" \
		"${GENERATOR_JAR}.tmp"
	mv "${GENERATOR_JAR}.tmp" "$GENERATOR_JAR"
fi

# Keep the spec in the repo so sync commits show the upstream API diff.
echo "==> Downloading OpenAPI spec (${FC_TAG})..."
fetch "https://raw.githubusercontent.com/${UPSTREAM_REPO}/${FC_TAG}/src/firecracker/swagger/firecracker.yaml" "${SPEC_FILE}.tmp"
mv "${SPEC_FILE}.tmp" "$SPEC_FILE"

echo "==> Generating SDK..."
java -jar "$GENERATOR_JAR" generate \
	-i "$SPEC_FILE" \
	-g rust \
	-o . \
	--additional-properties=packageName=firecracker-rust-sdk,packageVersion="${SDK_VERSION}"

echo "==> Updating version metadata..."
echo "$SDK_VERSION" >.firecracker-version
sed -i "0,/^version = \".*\"/s//version = \"${SDK_VERSION}\"/" Cargo.toml
sed -i "s|${UPSTREAM_REPO}/releases/download/v[0-9.]*/firecracker-v[0-9.]*-|${UPSTREAM_REPO}/releases/download/${FC_TAG}/firecracker-${FC_TAG}-|g" README.md

echo "==> Formatting code..."
cargo fmt

echo "==> Building to verify (including examples)..."
cargo build --all-targets

echo ""
echo "==> Regeneration complete: Firecracker ${FC_TAG} -> SDK v${SDK_VERSION}"
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Update src/lib.rs or src/machine.rs if the API changed"
echo "  3. Test: cargo clippy --all-targets -- -D warnings && cargo test"
