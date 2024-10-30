#! /bin/bash
# This script takes all dependencies that cargo vet critizes and checks with cargo tree if they are present in our dependency tree, if not they are added to the exemptions
OUTPUT=$(cargo vet --output-format json | jq -r '.failures.[] | select(.missing_criteria[0] == "safe-to-deploy") | "\(.name)@\(.version)"')
mapfile -t FAILURES <<< "$OUTPUT"
for FAILURE in "${FAILURES[@]}"
do
  if [ -z "$(cargo tree --quiet --all-features -e normal -i ${FAILURE})" ]; then
    echo "Ignoring dev/optional dependency $FAILURE"

    cargo vet add-exemption ${FAILURE//@/ }
  else
    echo "$FAILURE is normal dependency"
  fi
done

