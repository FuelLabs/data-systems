#!/bin/bash

# Function to get the cutoff date
get_date() {
    local days="$1"
    if ! CUTOFF_DATE=$(date --date="${days} days ago" +'%Y-%m-%dT%H:%M:%SZ' 2> /dev/null); then
        CUTOFF_DATE=$(date -v-"${days}"d +'%Y-%m-%dT%H:%M:%SZ' 2> /dev/null)
    fi
    echo "$CUTOFF_DATE"
}

# Check if required arguments are provided
if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <repo_owner> <repo_name> <days_to_keep>"
    exit 1
fi

REPO_OWNER="$1"
REPO_NAME="$2"
DAYS_TO_KEEP="$3"

# Ensure gh CLI is installed and authenticated
if ! command -v gh &> /dev/null; then
    echo "GitHub CLI (gh) is not installed. Please install it first."
    exit 1
fi

if ! gh auth status &> /dev/null; then
    echo "GitHub CLI is not authenticated. Please run 'gh auth login' first."
    exit 1
fi

# Get the cutoff date
CUTOFF_DATE=$(get_date "$DAYS_TO_KEEP")
echo "Cutoff date: $CUTOFF_DATE"

# Delete old artifacts
PAGE=1
while true; do
    echo "Processing page $PAGE"

    RESPONSE=$(gh api "repos/$REPO_OWNER/$REPO_NAME/actions/artifacts?per_page=100&page=$PAGE" 2>&1)
    if [[ $? -ne 0 ]]; then
        echo "Error fetching artifacts: $RESPONSE"
        exit 1
    fi

    ART_EXIST=$(echo "$RESPONSE" | jq -r '.artifacts[]')
    if [[ -z "$ART_EXIST" ]]; then
        echo "No more artifacts found."
        break
    fi

    ARTIFACTS=$(echo "$RESPONSE" | jq -r ".artifacts[] | select(.created_at < \"$CUTOFF_DATE\") | .id")

    for ARTIFACT_ID in $ARTIFACTS; do
        ARTIFACT_INFO=$(gh api "repos/$REPO_OWNER/$REPO_NAME/actions/artifacts/$ARTIFACT_ID" 2>&1)
        if [[ $? -ne 0 ]]; then
            echo "Error fetching artifact info for ID $ARTIFACT_ID: $ARTIFACT_INFO"
            continue
        fi

        ARTIFACT_NAME=$(echo "$ARTIFACT_INFO" | jq -r '.name')
        echo "Deleting artifact $ARTIFACT_NAME (ID: $ARTIFACT_ID)..."
        DELETE_RESULT=$(gh api "repos/$REPO_OWNER/$REPO_NAME/actions/artifacts/$ARTIFACT_ID" -X DELETE 2>&1)
        if [[ $? -ne 0 ]]; then
            echo "Error deleting artifact $ARTIFACT_NAME (ID: $ARTIFACT_ID): $DELETE_RESULT"
        else
            echo "Successfully deleted artifact $ARTIFACT_NAME (ID: $ARTIFACT_ID)"
        fi
    done

    PAGE=$((PAGE + 1))
done

echo "Artifact cleanup completed."
