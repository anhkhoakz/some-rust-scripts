FOLDERS := "aspect-ratio count-characters json-compare-cli"

# Lists all available just commands
default:
    just --list

# Prints usage information
help:
    @echo "usage: just [command]"

# Build and move binaries
build-all:
    for folder in aspect-ratio count-characters json-compare-cli; do \
        cd $folder; \
        # just build; \
        binary_name=$(rg -A 1 "\[package\]" Cargo.toml | rg "name\s*=\s*\"(.*)\"" -r '$1'); \
        if [ -f "target/release/$binary_name" ]; then \
            cp "target/release/$binary_name" ../tmp/; \
            echo "Copied $binary_name to ../tmp/"; \
        else \
            echo "Binary $binary_name not found in target/release/"; \
        fi; \
        cd ..; \
    done

# Remove all binaries
clean:
    git clean -Xdf
    fd --type=directory -HI 'target' -X rm -rf

