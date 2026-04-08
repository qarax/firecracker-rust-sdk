#!/bin/bash
set -e

# Regenerate firecracker-rust-sdk from the Firecracker OpenAPI spec
# The spec version is embedded in firecracker.yaml (info.version field)
# and is used to set the package version in Cargo.toml.

SPEC_FILE="firecracker.yaml"
SDK_NAME="firecracker-rust-sdk"

if [ ! -f "$SPEC_FILE" ]; then
    echo "ERROR: $SPEC_FILE not found. Run this script from the repo root."
    exit 1
fi

# Extract API version from the spec
API_VERSION=$(grep -m1 'version:' "$SPEC_FILE" | awk '{print $2}' | tr -d '"' | tr -d "'" | sed 's/-dev//')
if [ -z "$API_VERSION" ]; then
    echo "WARNING: Could not extract version from $SPEC_FILE, defaulting to 0.1.0"
    API_VERSION="0.1.0"
fi
echo "==> Firecracker API version: $API_VERSION"

echo "==> Fetching latest OpenAPI Generator version..."
if command -v curl > /dev/null 2>&1; then
    GENERATOR_VERSION=$(curl -sSL "https://api.github.com/repos/OpenAPITools/openapi-generator/releases/latest" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"v\([^"]*\)".*/\1/')
elif command -v wget > /dev/null 2>&1; then
    GENERATOR_VERSION=$(wget -qO- "https://api.github.com/repos/OpenAPITools/openapi-generator/releases/latest" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"v\([^"]*\)".*/\1/')
fi

if [ -z "$GENERATOR_VERSION" ]; then
    echo "    WARNING: Could not fetch latest generator version, falling back to 7.10.0"
    GENERATOR_VERSION="7.10.0"
fi
echo "    Latest OpenAPI Generator: v${GENERATOR_VERSION}"

GENERATOR_JAR="openapi-generator-cli.jar"
GENERATOR_URL="https://repo1.maven.org/maven2/org/openapitools/openapi-generator-cli/${GENERATOR_VERSION}/openapi-generator-cli-${GENERATOR_VERSION}.jar"

# Function to check if command exists
command_exists() {
    command -v "$1" > /dev/null 2>&1
}

# Function to get openapi-generator command
get_generator_cmd() {
    # Check if openapi-generator-cli is available via npm
    if command_exists openapi-generator-cli; then
        echo "openapi-generator-cli"
        return 0
    fi

    # Check if we have the JAR file locally and it matches the desired version
    if [ -f ".openapi-generator/$GENERATOR_JAR" ] && [ -f ".openapi-generator/generator-version" ]; then
        CACHED_VERSION=$(cat ".openapi-generator/generator-version")
        if [ "$CACHED_VERSION" = "$GENERATOR_VERSION" ]; then
            echo "java -jar .openapi-generator/$GENERATOR_JAR"
            return 0
        fi
        echo "==> Cached generator version ($CACHED_VERSION) differs from latest ($GENERATOR_VERSION), re-downloading..." >&2
    fi

    # Download the JAR file
    echo "==> Downloading OpenAPI Generator v${GENERATOR_VERSION}..." >&2
    mkdir -p .openapi-generator

    if command_exists curl; then
        curl -sSL "$GENERATOR_URL" -o ".openapi-generator/$GENERATOR_JAR"
    elif command_exists wget; then
        wget -q "$GENERATOR_URL" -O ".openapi-generator/$GENERATOR_JAR"
    else
        echo "ERROR: Neither curl nor wget found. Please install one of them." >&2
        exit 1
    fi

    echo "$GENERATOR_VERSION" > ".openapi-generator/generator-version"
    echo "java -jar .openapi-generator/$GENERATOR_JAR"
}

# Check for Java
if ! command_exists java; then
    echo "ERROR: Java is not installed. Please install Java JRE/JDK (version 8 or later)."
    echo ""
    echo "Installation options:"
    echo "  - Arch Linux: sudo pacman -S jre-openjdk"
    echo "  - Ubuntu/Debian: sudo apt install default-jre"
    echo "  - Fedora: sudo dnf install java-latest-openjdk"
    echo ""
    echo "Or install openapi-generator-cli via npm:"
    echo "  npm install -g @openapitools/openapi-generator-cli"
    exit 1
fi

GENERATOR_CMD=$(get_generator_cmd)

echo "==> Generating SDK with openapi-generator..."
echo "    Using: $GENERATOR_CMD"
$GENERATOR_CMD generate \
    -i "$SPEC_FILE" \
    -g rust \
    -o . \
    --additional-properties=packageName=${SDK_NAME},packageVersion=${API_VERSION}

echo "==> Writing version file..."
echo "$API_VERSION" > .firecracker-version

echo "==> Applying post-generation patches..."

# The generated Cargo.toml uses packageName as the crate name but we need the
# lib name to use underscores. Patch it to add an explicit [lib] section.
if ! grep -q '^\[lib\]' Cargo.toml; then
    # Insert [lib] section after [package]
    awk '/^\[dependencies\]/{print "[lib]"; print "name = \"firecracker_rust_sdk\""; print "path = \"src/lib.rs\""; print ""; print $0; next}1' Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml
    echo "    Patched Cargo.toml to add [lib] section"
fi

# Add extra dependencies not included by generator that our hand-crafted files need
if ! grep -q 'thiserror' Cargo.toml; then
    echo 'thiserror = "2"' >> Cargo.toml
fi
if ! grep -q 'uuid' Cargo.toml; then
    echo 'uuid = { version = "1.0", features = ["serde", "v4"] }' >> Cargo.toml
fi
if ! grep -q 'tracing' Cargo.toml; then
    echo 'tracing = "0.1"' >> Cargo.toml
fi
if ! grep -q 'pin-project-lite' Cargo.toml; then
    echo 'pin-project-lite = "0.2"' >> Cargo.toml
fi
if ! grep -q 'futures' Cargo.toml; then
    echo 'futures = "0.3"' >> Cargo.toml
fi
if ! grep -q 'http-body-util' Cargo.toml; then
    echo 'http-body-util = "0.1"' >> Cargo.toml
fi
if ! grep -q 'hyper' Cargo.toml; then
    echo 'hyper = { version = "1", features = ["client", "http1"] }' >> Cargo.toml
fi

# Enable serde's derive feature – required so that #[derive(Serialize)] works on
# local structs inside async functions in machine.rs.
sed -i 's/^serde = "[^"]*"/serde = { version = "1.0", features = ["derive"] }/' Cargo.toml
# If serde already has a features array, add "derive" if missing
sed -i '/^serde = {/{ /features/{ /"derive"/!s/\(features = \[\)/\1"derive", / } }' Cargo.toml
echo "    Ensured serde has derive feature"

# Overwrite lib.rs to expose our hand-crafted modules alongside generated ones
cat > src/lib.rs << 'EOF'
#![allow(unused_imports)]
#![allow(dead_code)]

#[macro_use]
extern crate serde_derive;

pub mod client;
pub mod error;
pub mod machine;
#[allow(clippy::empty_docs, clippy::derivable_impls)]
pub mod models;
EOF

echo "    Patched src/lib.rs"

echo "==> Copying hand-crafted source files..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# client.rs, error.rs, machine.rs are hand-crafted – copy them from the
# _handcrafted/ directory if they exist there (i.e. they were not overwritten
# by the generator) otherwise they were already in place.
for f in client.rs error.rs machine.rs; do
    if [ -f "_handcrafted/src/$f" ]; then
        cp "_handcrafted/src/$f" "src/$f"
        echo "    Restored src/$f from _handcrafted/"
    fi
done

echo "==> Formatting code..."
cargo fmt

echo "==> Building to verify..."
cargo build

echo ""
echo "==> Regeneration complete!"
echo "    Firecracker API version: $API_VERSION"
echo "    Custom files (machine.rs, client.rs, error.rs) were preserved."
echo "    Generated files (models/, src/lib.rs) have been updated."
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Tag the release: git tag v${API_VERSION}"
echo "  3. Test: cargo test"
