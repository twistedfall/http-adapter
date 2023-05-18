#!/bin/bash

set -e

README_TPL="$WORKSPACE_ROOT/README.tpl"
README="$CRATE_ROOT/README.md"

if [[ "$DRY_RUN" == "false" ]]; then
	cargo readme --template="$README_TPL" --output="$README"
	if [[ "$CRATE_NAME" == "http-adapter" ]]; then
		cp "$README" "$WORKSPACE_ROOT/README.md"
	fi
else
	echo "Dry run, would generate $README from $README_TPL"
fi
