#!/usr/bin/env python3
import os
import sys
import re
import json
import datetime
import tempfile
import logging
import requests
import concurrent.futures
import subprocess
from git import Repo, GitCommandError

GITHUB_GRAPHQL_URL = "https://api.github.com/graphql"

# ---------------------------
# Logging Configuration
# ---------------------------
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s"
)

# ---------------------------
# Helper Functions
# ---------------------------
def is_dry_run():
    """Check if DRY_RUN mode is enabled."""
    return os.environ.get("DRY_RUN", "").lower() in ("1", "true", "yes")

def configure_safe_directory(repo_path):
    """
    Configure Git to treat the repository directory as safe using GitPython's config writer.
    """
    try:
        with Repo(repo_path).config_writer(config_level='global') as cw:
            cw.set_value("safe", "directory", repo_path)
        logging.info("Configured safe.directory for %s", repo_path)
    except Exception as e:
        logging.error("Failed to configure safe.directory: %s", e)

def configure_git_identity(repo):
    """
    Configure a default Git identity (user.name and user.email) if not already set.
    """
    try:
        config_reader = repo.config_reader()
    except Exception:
        config_reader = None

    try:
        with repo.config_writer() as cw:
            try:
                name = config_reader.get_value("user", "name")
            except Exception:
                name = None
            try:
                email = config_reader.get_value("user", "email")
            except Exception:
                email = None
            if not name:
                cw.set_value("user", "name", "GitHub Actions")
                logging.info("Configured git user.name as 'GitHub Actions'")
            if not email:
                cw.set_value("user", "email", "actions@github.com")
                logging.info("Configured git user.email as 'actions@github.com'")
    except Exception as e:
        logging.error("Failed to configure git identity: %s", e)

def graphql_query(query, variables, token):
    headers = {"Authorization": f"Bearer {token}"}
    if is_dry_run():
        logging.info("[DRY RUN] Would execute GraphQL query with variables: %s", json.dumps(variables, indent=2))
        return {}  # Return an empty result in dry-run mode.
    try:
        response = requests.post(GITHUB_GRAPHQL_URL, json={"query": query, "variables": variables}, headers=headers)
        response.raise_for_status()
    except requests.RequestException as e:
        logging.error("GraphQL query failed: %s", e)
        sys.exit(1)
    result = response.json()
    if "errors" in result:
        logging.error("GraphQL query returned errors: %s", result["errors"])
        sys.exit(1)
    return result

def detect_changed_crates(repo):
    """
    Detect crates that changed by examining the diff of HEAD against its parent.
    If the diff fails, fallback to traversing the commit tree.
    Returns a sorted list of crate names (directories under "apps/").
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

def determine_bump_type():
    """
    Determine the version bump type.
    If the event is a pull_request, inspect its labels via GitHub GraphQL.
    Highest precedence: major > minor > patch.
    """
    bump_type = "patch"
    event_name = os.environ.get("GITHUB_EVENT_NAME", "")
    if event_name == "pull_request":
        event_path = os.environ.get("GITHUB_EVENT_PATH")
        if event_path and os.path.exists(event_path):
            with open(event_path) as f:
                event_data = json.load(f)
            pr_number = event_data.get("pull_request", {}).get("number")
            if pr_number:
                token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
                repo_full = os.environ.get("GITHUB_REPOSITORY")
                owner, repo_name = repo_full.split("/")
                query = '''
                query($owner: String!, $name: String!, $number: Int!) {
                  repository(owner: $owner, name: $name) {
                    pullRequest(number: $number) {
                      labels(first: 10) {
                        nodes {
                          name
                        }
                      }
                    }
                  }
                }
                '''
                variables = {"owner": owner, "name": repo_name, "number": pr_number}
                result = graphql_query(query, variables, token)
                labels = [node["name"] for node in result.get("data", {}).get("repository", {}) \
                    .get("pullRequest", {}).get("labels", {}).get("nodes", [])]
                logging.info("PR labels: %s", labels)
                if "major" in labels:
                    bump_type = "major"
                elif "minor" in labels:
                    bump_type = "minor"
    logging.info("Determined bump type: %s", bump_type)
    return bump_type

def push_with_token(repo, tag_name=None):
    """
    Use the Git CLI to update the remote URL with the token, push commits (and optionally tag),
    then reset the remote URL back to its original value.
    """
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    origin = repo.remotes.origin
    original_url = origin.url
    if token and original_url.startswith("https://"):
        new_url = original_url.replace("https://", f"https://{token}:x-oauth-basic@")
        try:
            subprocess.run(["git", "remote", "set-url", "origin", new_url], check=True)
        except Exception as e:
            logging.error("Error setting remote URL with token: %s", e)
    try:
        origin.push()
        if tag_name:
            origin.push(tag_name)
        logging.info("Pushed changes and tag %s", tag_name if tag_name else "")
    except GitCommandError as e:
        logging.error("Error pushing changes: %s", e)
    finally:
        # Reset the remote URL to the original value
        if token and original_url.startswith("https://"):
            try:
                subprocess.run(["git", "remote", "set-url", "origin", original_url], check=True)
            except Exception as e:
                logging.error("Error resetting remote URL: %s", e)

def update_crate(repo, crate, bump_type):
    """
    For the given crate, update Cargo.toml version and CHANGELOG.md,
    commit the changes, and create a new git tag.
    Returns the new version and created tag name.
    """
    crate_dir = os.path.join("apps", crate)
    cargo_toml_path = os.path.join(crate_dir, "Cargo.toml")
    if not os.path.isfile(cargo_toml_path):
        logging.error("Cargo.toml not found for %s", crate)
        return None, None

    with open(cargo_toml_path, "r") as f:
        cargo_content = f.read()
    m = re.search(r'^version\s*=\s*"(\d+\.\d+\.\d+)"', cargo_content, re.MULTILINE)
    if not m:
        logging.error("Version not found in %s", cargo_toml_path)
        return None, None
    current_version = m.group(1)
    major, minor, patch = map(int, current_version.split('.'))

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

    if is_dry_run():
        logging.info("[DRY RUN] Would update Cargo.toml for %s", crate)
        return new_version, f"{crate}-{new_version}"

    updated_cargo_content = re.sub(
        r'^(version\s*=\s*")(\d+\.\d+\.\d+)(")',
        lambda m: f'{m.group(1)}{new_version}{m.group(3)}',
        cargo_content,
        flags=re.MULTILINE
    )
    with open(cargo_toml_path, "w") as f:
        f.write(updated_cargo_content)

    changelog_path = os.path.join(crate_dir, "CHANGELOG.md")
    if not os.path.isfile(changelog_path):
        with open(changelog_path, "w") as f:
            f.write("# Changelog\n\n")
    try:
        tags = [t.name for t in repo.tags if t.name.startswith(f"{crate}-")]
        if tags:
            def version_key(tag):
                v = tag[len(crate)+1:].split("-")[0]
                return tuple(map(int, v.split('.')))
            latest_tag = sorted(tags, key=version_key)[-1]
        else:
            latest_tag = None
    except Exception:
        latest_tag = None

    commits = []
    try:
        if latest_tag:
            commits_iter = repo.iter_commits(rev=f"{latest_tag}..HEAD", paths=crate_dir)
        else:
            commits_iter = repo.iter_commits(paths=crate_dir)
        for commit in commits_iter:
            commits.append(f"* {commit.message.splitlines()[0]}")
    except Exception:
        commits = []
    if commits:
        date_str = datetime.datetime.now().strftime("%Y-%m-%d")
        new_entry = f"## {new_version} ({date_str})\n\n" + "\n".join(commits) + "\n\n"
        with open(changelog_path, "r") as f:
            lines = f.readlines()
        if len(lines) > 1:
            lines.insert(1, new_entry)
        else:
            lines.append(new_entry)
        with open(changelog_path, "w") as f:
            f.writelines(lines)
        logging.info("Updated CHANGELOG.md for %s", crate)
    else:
        logging.info("No new commits for %s changelog update", crate)

    repo.index.add([cargo_toml_path, changelog_path])
    commit_msg = f"chore: bump {crate} version to {new_version} [skip ci]"
    repo.index.commit(commit_msg)
    logging.info("Committed changes for %s", crate)

    base_tag = f"{crate}-{new_version}"
    tag_name = base_tag
    counter = 1
    existing_tags = {t.name for t in repo.tags}
    while tag_name in existing_tags:
        tag_name = f"{base_tag}-{counter}"
        counter += 1
    repo.create_tag(tag_name, message=f"Release {crate} {new_version}")
    logging.info("Created tag %s for %s", tag_name, crate)

    if is_dry_run():
        logging.info("[DRY RUN] Would push commit and tag for %s", crate)
        return new_version, tag_name

    push_with_token(repo, tag_name)
    return new_version, tag_name

def update_downstream(crate, new_version, tag_name):
    """
    For the updated crate, check its Cargo.toml for a repository URL.
    If found, use GraphQL to fetch the Dockerfile from that downstream repository,
    update image references (including sha-* references), and create a PR if needed.
    Returns a dict with downstream update info if a PR was created.
    """
    crate_dir = os.path.join("apps", crate)
    cargo_toml_path = os.path.join(crate_dir, "Cargo.toml")
    with open(cargo_toml_path, "r") as f:
        cargo_content = f.read()
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

    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    query = '''
    query($owner: String!, $name: String!) {
      repository(owner: $owner, name: $name) {
        dockerfile: object(expression: "HEAD:Dockerfile") {
          ... on Blob {
             text
          }
        }
      }
    }
    '''
    variables = {"owner": owner, "name": repo_name}
    result = graphql_query(query, variables, token)
    docker_obj = result.get("data", {}).get("repository", {}).get("dockerfile")
    if not docker_obj or "text" not in docker_obj or not docker_obj["text"]:
        logging.info("No Dockerfile found in repository %s", repo_full)
        return None
    docker_text = docker_obj["text"]

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
    with open(dockerfile_path, "w") as f:
        f.write(new_docker_text)
    downstream_repo.index.add(["Dockerfile"])
    commit_msg = f"chore: update {crate} to version {new_version}"
    downstream_repo.index.commit(commit_msg)
    try:
        origin = downstream_repo.remotes.origin
        origin.push(refspec=f"{pr_branch}:{pr_branch}")
        logging.info("Pushed branch %s to %s", pr_branch, repo_full)
    except GitCommandError as e:
        logging.error("Error pushing downstream changes for %s: %s", repo_full, e)
        return None

    pr_title = f"Upgrading GSM Version to: {crate}-{new_version}"
    headers = {"Authorization": f"token {token}"}
    pr_list_url = f"https://api.github.com/repos/{repo_full}/pulls?state=open"
    r = requests.get(pr_list_url, headers=headers)
    if r.status_code != 200:
        logging.error("Failed to get PR list for %s: %s", repo_full, r.text)
        return None
    prs = r.json()
    for pr in prs:
        if pr.get("title") == pr_title:
            logging.info("PR already exists for %s: #%s", repo_full, pr.get('number'))
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
    r = requests.post(pr_url, headers=headers, json=payload)
    if r.status_code in [200, 201]:
        pr_data = r.json()
        logging.info("Created PR #%s for %s", pr_data.get("number"), repo_full)
        return {
            "crate": crate,
            "repo": repo_full,
            "new_version": new_version,
            "pr_number": pr_data.get("number"),
            "pr_url": pr_data.get("html_url")
        }
    else:
        logging.error("Failed to create PR for %s: %s", repo_full, r.text)
    return None

def append_summary(summary_updates):
    """
    Append summary information about downstream updates to GITHUB_STEP_SUMMARY.
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

# ---------------------------
# Main Flow
# ---------------------------
def main():
    repo = Repo(os.getcwd())
    configure_safe_directory(repo.working_dir)
    configure_git_identity(repo)

    changed_crates = detect_changed_crates(repo)
    if not changed_crates:
        logging.info("No changed crates detected. Exiting.")
        return

    bump_type = determine_bump_type()
    updated = []  # List of tuples: (crate, new_version, tag)

    for crate in changed_crates:
        new_version, tag_name = update_crate(repo, crate, bump_type)
        if new_version and tag_name:
            updated.append((crate, new_version, tag_name))

    downstream_updates = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=5) as executor:
        future_to_crate = {
            executor.submit(update_downstream, crate, new_version, tag): crate
            for crate, new_version, tag in updated
        }
        for future in concurrent.futures.as_completed(future_to_crate):
            update_info = future.result()
            if update_info:
                downstream_updates.append(update_info)

    if downstream_updates:
        append_summary(downstream_updates)
    else:
        logging.info("No downstream updates were performed.")

if __name__ == '__main__':
    main()
