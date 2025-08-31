#!/bin/bash
# Prune local branches that have been merged into origin/main

# Fetch the latest branches
echo "Fetching the latest branches..."
git fetch -p

# Get the list of merged branches
echo "Getting the list of merged branches..."
merged_branches=$(git branch --merged)

# Exclude certain branches from the list
echo "Excluding certain branches from the list..."
prunable_branches=$(echo "$merged_branches" | grep -v '*' | grep -v 'main' | grep -v 'dev')

# Check if there are any branches to prune
if [ -z "$prunable_branches" ]
then
    echo "No branches to prune."
else
    # Prune the branches
    echo "Pruning the branches..."
    echo "$prunable_branches" | xargs git branch -d
fi

echo
echo "All branches:"
git branch -a
echo
echo "Done."
