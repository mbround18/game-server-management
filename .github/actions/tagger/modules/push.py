"""
push.py

This module provides functions for pushing Git changes using an authentication token.
It temporarily updates the remote URL to include the token for authentication,
executes the push using subprocess (thus avoiding shell injection risks),
and resets the remote URL back to its original state.

Usage:
    from push import push_with_token
    push_with_token(repo, tag_name)  # repo is a GitPython Repo object
"""

import os
import subprocess
import logging

def push_with_token(repo, tag_name=None):
    """
    Push commits (and optionally a tag) to the remote repository using an authentication token.

    This function:
      1. Retrieves the token from the environment (GH_TOKEN or GITHUB_TOKEN).
      2. Temporarily updates the remote URL to include the token.
      3. Uses the Git CLI (via subprocess) to push commits and the tag.
      4. Resets the remote URL to its original value.

    Parameters:
        repo (git.Repo): The GitPython Repo object representing the repository.
        tag_name (str, optional): The name of the tag to push. If None, only commits are pushed.

    Returns:
        None
    """
    # Retrieve the authentication token
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if not token:
        logging.error("Authentication token not found in GH_TOKEN or GITHUB_TOKEN.")
        return

    # Retrieve the remote (origin) and its original URL
    origin = repo.remotes.origin
    original_url = origin.url

    # Ensure the remote URL uses HTTPS so we can inject the token.
    if original_url.startswith("https://"):
        tokenized_url = original_url.replace("https://", f"https://{token}:x-oauth-basic@")
    else:
        logging.error("Remote URL does not start with 'https://': %s", original_url)
        return

    try:
        # Push commits using the tokenized URL.
        subprocess.run(["git", "push", tokenized_url], check=True)
        logging.info("Pushed commits using tokenized URL.")
        # If a tag name is provided, push the tag as well.
        if tag_name:
            subprocess.run(["git", "push", tokenized_url, tag_name], check=True)
            logging.info("Pushed tag: %s", tag_name)
    except subprocess.CalledProcessError as e:
        logging.error("Error pushing changes: %s", e)
    finally:
        try:
            # Reset the remote URL back to its original value.
            subprocess.run(["git", "remote", "set-url", "origin", original_url], check=True)
            logging.info("Reset remote URL to original: %s", original_url)
        except subprocess.CalledProcessError as e:
            logging.error("Error resetting remote URL: %s", e)
