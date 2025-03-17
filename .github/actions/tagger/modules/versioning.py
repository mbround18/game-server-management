"""
versioning.py

This module is responsible for all versioning operations, including:

  - Detecting which crates (subdirectories under "apps/") have changed.
  - Updating the version number in Cargo.toml and the CHANGELOG.md for a crate.
  - Committing the changes with a "[skip ci]" message and creating a new Git tag.
  - Updating downstream dependencies (e.g. updating Dockerfiles in dependent repositories)
    via GitHub API calls.
  - Appending a summary of downstream updates to a file (if specified).

Each function logs its operations to help with debugging and maintainability.
"""

import os
import re
import datetime
import logging
from git import GitCommandError

def detect_changed_crates(repo):
    """
    Detect changed crates by examining the diff of HEAD against its parent.
    If diff computation fails, it falls back to traversing the commit tree.

    Parameters:
        repo (git.Repo): A GitPython repository object.

    Returns:
        List[str]: A sorted list of crate names (i.e. subdirectories under "apps/") that have changed.
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
            changed_files = [item.path for item in commit.tree.traverse() if item.type == 'blob']
    else:
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
    commit the changes (with a commit message that includes "[skip ci]"),
    and create a Git tag.

    Parameters:
        repo (git.Repo): A GitPython repository object.
        crate (str): The crate name (i.e. subdirectory in "apps/").
        bump_type (str): The type of version bump ("major", "minor", or "patch").

    Returns:
        Tuple[str, str]: A tuple (new_version, tag_name) or (None, None) on failure.
    """
    crate_dir = os.path.join("apps", crate)
    cargo_toml_path = os.path.join(crate_dir, "Cargo.toml")
    if not os.path.isfile(cargo_toml_path):
        logging.error("Cargo.toml not found for %s", crate)
        return None, None

    # Read Cargo.toml and extract the current version.
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

    # Determine new version.
    if bump_type == "major":
        major += 1; minor = 0; patch = 0
    elif bump_type == "minor":
        minor += 1; patch = 0
    else:
        patch += 1
    new_version = f"{major}.{minor}.{patch}"
    logging.info("Updating %s from version %s to %s", crate, current_version, new_version)

    # Update Cargo.toml.
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

    commits = []
    try:
        # If there are tags, only consider commits since the latest tag.
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
        logging.info("No new commits for %s; CHANGELOG.md remains unchanged", crate)

    # Commit changes with [skip ci] in the message.
    try:
        repo.index.add([cargo_toml_path, changelog_path])
        commit_msg = f"chore: bump {crate} version to {new_version} [skip ci]"
        repo.index.commit(commit_msg)
        logging.info("Committed changes for %s", crate)
    except Exception as e:
        logging.error("Error committing changes for %s: %s", crate, e)
        return None, None

    # Create a new tag.
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

def update_downstream(crate, new_version, tag_name):
    """
    Update downstream dependencies for a given crate.

    This function:
      - Reads the repository URL from the crate's Cargo.toml.
      - Uses a GraphQL query to fetch the Dockerfile from the downstream repository.
      - Updates image references (including sha-* references) with the new version.
      - Commits and pushes the change to a new branch and creates a PR if one does not already exist.

    Parameters:
        crate (str): The crate name.
        new_version (str): The new version string.
        tag_name (str): The created Git tag for the crate.

    Returns:
        dict: A dictionary containing downstream update details (crate, repo, new_version, pr_number, pr_url)
              if a PR was created; otherwise, None.
    """
    crate_dir = os.path.join("apps", crate)
    cargo_toml_path = os.path.join(crate_dir, "Cargo.toml")
    try:
        with open(cargo_toml_path, "r") as f:
            cargo_content = f.read()
    except Exception as e:
        logging.error("Error reading %s: %s", cargo_toml_path, e)
        return None

    m = re.search(r'^repository\s*=\s*"([^"]+)"', cargo_content, re.MULTILINE)
    if not m:
        logging.info("No repository URL found in %s for %s. Skipping downstream update.", cargo_toml_path, crate)
        return None
    repo_url = m.group(1)
    m_repo = re.search(r'https://github.com/([^/]+/[^/]+)', repo_url)
    if not m_repo:
        logging.info("Repository URL not valid for %s. Skipping downstream update.", crate)
        return None
    repo_full = m_repo.group(1)
    owner, repo_name = repo_full.split("/")
    logging.info("Found downstream repository for %s: %s", crate, repo_full)

    # Use the GitHub GraphQL API to fetch the Dockerfile content.
    from modules.github_utils import graphql_query
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    query = """
    query($owner: String!, $name: String!) {
      repository(owner: $owner, name: $name) {
        dockerfile: object(expression: "HEAD:Dockerfile") {
          ... on Blob {
            text
          }
        }
      }
    }
    """
    variables = {"owner": owner, "name": repo_name}
    result = graphql_query(query, variables, token)
    docker_obj = result.get("data", {}).get("repository", {}).get("dockerfile")
    if not docker_obj or "text" not in docker_obj or not docker_obj["text"]:
        logging.info("No Dockerfile found in repository %s", repo_full)
        return None
    docker_text = docker_obj["text"]

    # Update the image reference in the Dockerfile.
    pattern = fr"mbround18/gsm-reference:{crate}-([0-9]+\.[0-9]+\.[0-9]+)"
    m_docker = re.search(pattern, docker_text)
    if not m_docker:
        logging.info("Dockerfile in %s does not reference %s image", repo_full, crate)
        return None
    current_docker_version = m_docker.group(1)
    logging.info("Downstream Dockerfile current version for %s: %s", crate, current_docker_version)
    if current_docker_version == new_version:
        logging.info("Dockerfile already uses the latest version for %s", crate)
        return None

    new_docker_text = re.sub(
        fr"mbround18/gsm-reference:{crate}-[0-9.]+",
        f"mbround18/gsm-reference:{crate}-{new_version}",
        docker_text
    )
    new_docker_text = re.sub(
        fr"mbround18/gsm-reference:sha-[0-9a-f]+",
        f"mbround18/gsm-reference:{crate}-{new_version}",
        new_docker_text
    )

    # Clone the downstream repository, update the Dockerfile, and push changes.
    import tempfile
    from git import Repo, GitCommandError
    temp_dir = tempfile.mkdtemp(prefix="downstream_")
    clone_url = f"https://{token}:x-oauth-basic@github.com/{repo_full}.git"
    try:
        downstream_repo = Repo.clone_from(clone_url, temp_dir)
    except Exception as e:
        logging.error("Failed to clone repository %s: %s", repo_full, e)
        return None

    pr_branch = f"update-{crate}-to-{new_version}"
    try:
        downstream_repo.git.checkout('-b', pr_branch)
    except GitCommandError as e:
        logging.error("Error creating branch in %s: %s", repo_full, e)
        return None

    dockerfile_path = os.path.join(temp_dir, "Dockerfile")
    if not os.path.isfile(dockerfile_path):
        logging.info("Dockerfile not found after cloning %s", repo_full)
        return None
    try:
        with open(dockerfile_path, "w") as f:
            f.write(new_docker_text)
    except Exception as e:
        logging.error("Error writing updated Dockerfile for %s: %s", repo_full, e)
        return None

    try:
        downstream_repo.index.add(["Dockerfile"])
        commit_msg = f"chore: update {crate} to version {new_version}"
        downstream_repo.index.commit(commit_msg)
    except Exception as e:
        logging.error("Error committing Dockerfile changes for %s: %s", repo_full, e)
        return None

    try:
        origin = downstream_repo.remotes.origin
        origin.push(refspec=f"{pr_branch}:{pr_branch}")
        logging.info("Pushed branch %s to %s", pr_branch, repo_full)
    except GitCommandError as e:
        logging.error("Error pushing downstream changes for %s: %s", repo_full, e)
        return None

    # Create a pull request via the GitHub REST API.
    headers = {"Authorization": f"token {token}"}
    pr_list_url = f"https://api.github.com/repos/{repo_full}/pulls?state=open"
    try:
        r = requests.get(pr_list_url, headers=headers)
        r.raise_for_status()
    except Exception as e:
        logging.error("Failed to get PR list for %s: %s", repo_full, e)
        return None

    prs = r.json()
    pr_title = f"Upgrading GSM Version to: {crate}-{new_version}"
    for pr in prs:
        if pr.get("title") == pr_title:
            logging.info("PR already exists for %s: #%s", repo_full, pr.get("number"))
            return {
                "crate": crate,
                "repo": repo_full,
                "new_version": new_version,
                "pr_number": pr.get("number"),
                "pr_url": pr.get("html_url")
            }

    pr_url = f"https://api.github.com/repos/{repo_full}/pulls"
    payload = {
        "title": pr_title,
        "body": f"Updating to the latest version of {crate}: {new_version}",
        "head": pr_branch,
        "base": "main"
    }
    try:
        r = requests.post(pr_url, headers=headers, json=payload)
        r.raise_for_status()
    except Exception as e:
        logging.error("Failed to create PR for %s: %s", repo_full, e)
        return None

    pr_data = r.json()
    logging.info("Created PR #%s for %s", pr_data.get("number"), repo_full)
    return {
        "crate": crate,
        "repo": repo_full,
        "new_version": new_version,
        "pr_number": pr_data.get("number"),
        "pr_url": pr_data.get("html_url")
    }

def append_summary(summary_updates):
    """
    Append summary information about downstream updates to GITHUB_STEP_SUMMARY.

    If the GITHUB_STEP_SUMMARY environment variable is defined, this function
    appends a Markdown-formatted summary to that file.
    """
    summary_file = os.environ.get("GITHUB_STEP_SUMMARY")
    if not summary_file:
        logging.info("GITHUB_STEP_SUMMARY not defined, skipping summary update.")
        return
    summary_lines = ["## Downstream Repository Updates\n"]
    for update in summary_updates:
        summary_lines.append(
            f"- **Crate**: {update['crate']} | **Repo**: [{update['repo']}](https://github.com/{update['repo']}) | **New Version**: {update['new_version']} | **PR**: [#{update['pr_number']}]({update['pr_url']})"
        )
    try:
        with open(summary_file, "a") as f:
            f.write("\n".join(summary_lines) + "\n")
        logging.info("Appended downstream update summary to %s", summary_file)
    except Exception as e:
        logging.error("Error appending to summary file: %s", e)
