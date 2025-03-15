"""
versioning.py

This module is responsible for:
  - Detecting which crates (subdirectories under "apps/") have changed.
  - Updating the version number in Cargo.toml and the CHANGELOG.md file for a given crate.
  - Committing the changes with a "[skip ci]" message and creating a new Git tag.

Each function logs its key actions and errors to assist in debugging.
"""

import os
import re
import datetime
import logging
from git import GitCommandError

def detect_changed_crates(repo):
    """
    Detect changed crates by examining the diff of HEAD against its parent.
    If the diff cannot be computed, it falls back to traversing the commit tree.

    Parameters:
        repo (git.Repo): A GitPython repository object.

    Returns:
        List[str]: A sorted list of crate names (i.e. subdirectories in "apps/") that have changed.
    """
    commit = repo.head.commit
    changed_files = []
    if commit.parents:
        parent = commit.parents[0]
        try:
            diff_index = commit.diff(parent)
            changed_files = [item.a_path for item in diff_index if item.a_path]
        except GitCommandError as e:
            logging.error("Error computing diff via commit.diff: %s", e)
            # Fallback: Traverse the commit tree.
            changed_files = [item.path for item in commit.tree.traverse() if item.type == 'blob']
    else:
        # If this is the initial commit, include all files.
        changed_files = [item.path for item in commit.tree.traverse() if item.type == 'blob']

    crates = set()
    for f in changed_files:
        if f.startswith("apps/"):
            parts = f.split('/')
            if len(parts) >= 2:
                crates.add(parts[1])
    crates_list = sorted(crates)
    logging.info("Detected changed crates: %s", crates_list)
    return crates_list

def update_crate(repo, crate, bump_type):
    """
    Update the version for a given crate by modifying Cargo.toml and CHANGELOG.md,
    commit the changes with a message that includes "[skip ci]", and create a Git tag.

    Parameters:
        repo (git.Repo): A GitPython repository object.
        crate (str): The name of the crate (subdirectory under "apps/").
        bump_type (str): The type of version bump ("major", "minor", or "patch").

    Returns:
        Tuple[str, str]: A tuple containing the new version and the created tag name.
                         Returns (None, None) if an error occurs.
    """
    crate_dir = os.path.join("apps", crate)
    cargo_toml_path = os.path.join(crate_dir, "Cargo.toml")

    if not os.path.isfile(cargo_toml_path):
        logging.error("Cargo.toml not found for %s", crate)
        return None, None

    # Read Cargo.toml content and extract the current version.
    try:
        with open(cargo_toml_path, "r") as f:
            content = f.read()
    except Exception as e:
        logging.error("Error reading %s: %s", cargo_toml_path, e)
        return None, None

    match = re.search(r'^version\s*=\s*"(\d+\.\d+\.\d+)"', content, re.MULTILINE)
    if not match:
        logging.error("Version not found in %s", cargo_toml_path)
        return None, None
    current_version = match.group(1)
    major, minor, patch = map(int, current_version.split('.'))

    # Compute new version based on bump type.
    if bump_type == "major":
        major += 1
        minor = 0
        patch = 0
    elif bump_type == "minor":
        minor += 1
        patch = 0
    else:
        patch += 1
    new_version = f"{major}.{minor}.{patch}"
    logging.info("Updating %s from version %s to %s", crate, current_version, new_version)

    # Update Cargo.toml with the new version.
    updated_content = re.sub(
        r'^(version\s*=\s*")(\d+\.\d+\.\d+)(")',
        lambda m: f'{m.group(1)}{new_version}{m.group(3)}',
        content,
        flags=re.MULTILINE
    )
    try:
        with open(cargo_toml_path, "w") as f:
            f.write(updated_content)
    except Exception as e:
        logging.error("Error writing updated Cargo.toml for %s: %s", crate, e)
        return None, None

    # Update CHANGELOG.md.
    changelog_path = os.path.join(crate_dir, "CHANGELOG.md")
    if not os.path.isfile(changelog_path):
        try:
            with open(changelog_path, "w") as f:
                f.write("# Changelog\n\n")
        except Exception as e:
            logging.error("Error creating CHANGELOG.md for %s: %s", crate, e)
            return None, None

    # Retrieve commits for the crate.
    commits = []
    try:
        tags = [t.name for t in repo.tags if t.name.startswith(f"{crate}-")]
        if tags:
            def version_key(tag):
                v = tag[len(crate)+1:].split("-")[0]
                return tuple(map(int, v.split('.')))
            latest_tag = sorted(tags, key=version_key)[-1]
            commits_iter = repo.iter_commits(rev=f"{latest_tag}..HEAD", paths=crate_dir)
        else:
            commits_iter = repo.iter_commits(paths=crate_dir)
        for commit in commits_iter:
            commits.append(f"* {commit.message.splitlines()[0]}")
    except Exception as e:
        logging.error("Error collecting commit messages for %s: %s", crate, e)
        commits = []

    if commits:
        date_str = datetime.datetime.now().strftime("%Y-%m-%d")
        new_entry = f"## {new_version} ({date_str})\n\n" + "\n".join(commits) + "\n\n"
        try:
            with open(changelog_path, "r") as f:
                lines = f.readlines()
            if len(lines) > 1:
                lines.insert(1, new_entry)
            else:
                lines.append(new_entry)
            with open(changelog_path, "w") as f:
                f.writelines(lines)
            logging.info("Updated CHANGELOG.md for %s", crate)
        except Exception as e:
            logging.error("Error updating CHANGELOG.md for %s: %s", crate, e)
    else:
        logging.info("No new commits for %s; not updating CHANGELOG.md", crate)

    # Stage changes and commit with "[skip ci]".
    try:
        repo.index.add([cargo_toml_path, changelog_path])
        commit_msg = f"chore: bump {crate} version to {new_version} [skip ci]"
        repo.index.commit(commit_msg)
        logging.info("Committed changes for %s", crate)
    except Exception as e:
        logging.error("Error committing changes for %s: %s", crate, e)
        return None, None

    # Create a new Git tag, ensuring uniqueness.
    base_tag = f"{crate}-{new_version}"
    tag_name = base_tag
    counter = 1
    existing_tags = {t.name for t in repo.tags}
    while tag_name in existing_tags:
        tag_name = f"{base_tag}-{counter}"
        counter += 1
    try:
        repo.create_tag(tag_name, message=f"Release {crate} {new_version}")
        logging.info("Created tag %s for %s", tag_name, crate)
    except Exception as e:
        logging.error("Error creating tag for %s: %s", crate, e)
        return None, None

    return new_version, tag_name
