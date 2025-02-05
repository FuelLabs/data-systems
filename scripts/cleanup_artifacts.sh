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

# Debug: Print input parameters using bun
echo "Debug: Script parameters (using bun):"
echo "  REPO_OWNER: ${REPO_OWNER}"
echo "  REPO_NAME: ${REPO_NAME}"
echo "  DAYS_TO_KEEP: ${DAYS_TO_KEEP}"

# Ensure gh CLI is installed and authenticated
if ! command -v gh &> /dev/null; then
    echo "GitHub CLI (gh) is not installed. Please install it first."
    exit 1
fi

# Debug: Check gh auth status
echo "Debug: Checking gh auth status..."
gh auth status

# Get the cutoff date
CUTOFF_DATE=$(get_date "$DAYS_TO_KEEP")
echo "Cutoff date: $CUTOFF_DATE"

# Delete old artifacts
PAGE=1
while true; do
    echo "Processing page $PAGE"

    # Debug: Print API URL being called
    API_URL="repos/$REPO_OWNER/$REPO_NAME/actions/artifacts?per_page=100&page=$PAGE"
    echo "Debug: Calling API endpoint: $API_URL"

    # Debug: Test API call with curl
    echo "Debug: Testing API endpoint with curl..."
    curl -s -I -H "Authorization: Bearer $GITHUB_TOKEN" "https://api.github.com/$API_URL"

    RESPONSE=$(gh api "$API_URL" 2>&1)
    if [[ $? -ne 0 ]]; then
        echo "Error fetching artifacts: $RESPONSE"
        echo "Debug: Full API response:"
        echo "$RESPONSE"
        exit 1
    fi

    # Debug: Print response structure (without sensitive data)
    echo "Debug: Response structure:"
    echo "$RESPONSE" | jq 'del(.artifacts[].archive_download_url)' 2> /dev/null || echo "Failed to parse response as JSON"

    ART_EXIST=$(echo "$RESPONSE" | jq -r '.artifacts[]')
    if [[ -z "$ART_EXIST" ]]; then
        echo "No more artifacts found."
        break
    fi

    ARTIFACTS=$(echo "$RESPONSE" | jq -r ".artifacts[] | select(.created_at < \"$CUTOFF_DATE\") | .id")

    for ARTIFACT_ID in $ARTIFACTS; do
        echo "Debug: Processing artifact ID: $ARTIFACT_ID"
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
