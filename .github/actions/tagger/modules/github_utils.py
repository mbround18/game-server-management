"""
github_utils.py

This module provides utility functions to interact with GitHub, including:
  - Executing GraphQL queries against the GitHub GraphQL API.
  - Determining the version bump type (major, minor, or patch) based on pull request labels.
  - Checking if the script is running in dry-run mode.

These functions are used by the main automation scripts to fetch event data
and decide how to bump version numbers in the repository.
"""

import os
import json
import logging
import requests

# The GitHub GraphQL API endpoint.
GITHUB_GRAPHQL_URL = "https://api.github.com/graphql"

def is_dry_run():
    """
    Check if the script should run in dry-run mode.

    Returns:
        bool: True if the DRY_RUN environment variable is set to a truthy value, False otherwise.
    """
    return os.environ.get("DRY_RUN", "").lower() in ("1", "true", "yes")

def graphql_query(query, variables, token):
    """
    Execute a GraphQL query against the GitHub GraphQL API.

    Parameters:
        query (str): The GraphQL query string.
        variables (dict): Dictionary of variables to pass into the query.
        token (str): The GitHub authentication token (GH_TOKEN or GITHUB_TOKEN).

    Returns:
        dict: The JSON response from the API.

    Raises:
        Exception: If the API call fails or if the response contains errors.
    """
    headers = {"Authorization": f"Bearer {token}"}
    if is_dry_run():
        logging.info("[DRY RUN] Would execute GraphQL query with variables:\n%s",
                     json.dumps(variables, indent=2))
        return {}  # Return an empty result in dry-run mode.
    try:
        response = requests.post(GITHUB_GRAPHQL_URL, json={"query": query, "variables": variables}, headers=headers)
        response.raise_for_status()
    except requests.RequestException as e:
        logging.error("GraphQL query failed: %s", e)
        raise
    result = response.json()
    if "errors" in result:
        logging.error("GraphQL query returned errors: %s", result["errors"])
        raise Exception("GraphQL query error")
    return result

def determine_bump_type():
    """
    Determine the version bump type (major, minor, or patch) based on pull request labels.

    This function reads GitHub event data from the file specified by GITHUB_EVENT_PATH
    (if running as a pull_request event) and uses the GitHub GraphQL API to retrieve the PR labels.
    If a label "major" is present, it returns "major"; if "minor" is present, it returns "minor";
    otherwise, it defaults to "patch".

    Returns:
        str: The bump type ("major", "minor", or "patch").
    """
    bump_type = "patch"
    event_name = os.environ.get("GITHUB_EVENT_NAME", "")

    if event_name == "pull_request":
        event_path = os.environ.get("GITHUB_EVENT_PATH")
        if event_path and os.path.exists(event_path):
            try:
                with open(event_path) as f:
                    event_data = json.load(f)
            except Exception as e:
                logging.error("Error reading GitHub event data: %s", e)
                return bump_type

            pr_number = event_data.get("pull_request", {}).get("number")
            if pr_number:
                token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
                repo_full = os.environ.get("GITHUB_REPOSITORY")
                if not repo_full:
                    logging.error("GITHUB_REPOSITORY is not set in the environment.")
                    return bump_type
                owner, repo_name = repo_full.split("/")
                query = """
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
                """
                variables = {"owner": owner, "name": repo_name, "number": pr_number}
                try:
                    result = graphql_query(query, variables, token)
                except Exception as e:
                    logging.error("GraphQL query for bump type failed: %s", e)
                    return bump_type

                labels = [node["name"] for node in result.get("data", {}).get("repository", {}) \
                    .get("pullRequest", {}).get("labels", {}).get("nodes", [])]
                logging.info("Retrieved PR labels: %s", labels)
                if "major" in labels:
                    bump_type = "major"
                elif "minor" in labels:
                    bump_type = "minor"
    logging.info("Determined bump type: %s", bump_type)
    return bump_type
