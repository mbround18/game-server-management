#!/usr/bin/env python3
"""
main.py

This is the entry point for the automated versioning workflow. It performs the following:
  1. Configures the Git repository for safe operations and sets a default identity.
  2. Detects changed crates (subdirectories under "apps/") using GitPython.
  3. Determines the appropriate version bump (major, minor, or patch) from GitHub event data.
  4. For each changed crate, updates its Cargo.toml and CHANGELOG.md, commits the change (with [skip ci] in the commit message), and creates a Git tag.
  5. Processes downstream updates concurrently (e.g., updating Dockerfiles in dependent repos) and appends a summary if required.

Usage:
    # For a dry run (no actual pushes/changes):
    DRY_RUN=1 python main.py

    # For normal operation:
    python main.py
"""

import os
import logging
import concurrent.futures
from git import Repo

# Import functions from our modularized code.
from modules.config import configure_safe_directory, configure_git_identity
from modules.github_utils import determine_bump_type
from modules.versioning import detect_changed_crates, update_crate, update_downstream, append_summary

# Set up basic logging (this can also be configured in a separate logging config file)
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s"
)

def main():
    logging.info("Starting automated versioning workflow...")

    # Open the repository from the current working directory.
    repo = Repo(os.getcwd())

    # Configure the repository: mark as safe and set a default Git identity.
    configure_safe_directory(repo.working_dir)
    configure_git_identity(repo)

    # Detect which crates (subdirectories under "apps/") have changed.
    changed_crates = detect_changed_crates(repo)
    if not changed_crates:
        logging.info("No changed crates detected. Exiting workflow.")
        return

    # Determine the version bump type based on PR labels (major > minor > patch).
    bump_type = determine_bump_type()
    logging.info("Version bump type determined: %s", bump_type)

    # For each changed crate, update version (in Cargo.toml and CHANGELOG.md), commit, and create a tag.
    updated = []  # Will hold tuples of (crate, new_version, tag_name)
    for crate in changed_crates:
        new_version, tag_name = update_crate(repo, crate, bump_type)
        if new_version and tag_name:
            logging.info("Crate %s updated to version %s (tag: %s)", crate, new_version, tag_name)
            updated.append((crate, new_version, tag_name))
        else:
            logging.error("Failed to update version for crate: %s", crate)

    # Process downstream updates concurrently.
    downstream_updates = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=5) as executor:
        # Submit an update_downstream task for each updated crate.
        future_to_crate = {
            executor.submit(update_downstream, crate, new_version, tag): crate
            for crate, new_version, tag in updated
        }
        for future in concurrent.futures.as_completed(future_to_crate):
            update_info = future.result()
            if update_info:
                downstream_updates.append(update_info)

    # If downstream updates occurred, append a summary (e.g. for GitHub Step Summary).
    if downstream_updates:
        append_summary(downstream_updates)
    else:
        logging.info("No downstream updates were performed.")

    logging.info("Automated versioning workflow completed.")

if __name__ == '__main__':
    main()
