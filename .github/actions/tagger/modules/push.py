import os
import subprocess
import logging

def push_with_token(repo, tag_name=None):
    """
    Push commits (and optionally a tag) to the remote repository using an authentication token.
    This function:
      1. Retrieves the token from the environment.
      2. Temporarily updates the remote URL to include the token.
      3. Pushes commits and, if provided, the specific tag.
      4. Resets the remote URL back to its original value.
    """
    # Retrieve the authentication token.
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if not token:
        logging.error("Authentication token not found in GH_TOKEN or GITHUB_TOKEN.")
        return

    # Get the original remote URL.
    origin = repo.remotes.origin
    original_url = origin.url

    if not original_url.startswith("https://"):
        logging.error("Remote URL must use HTTPS: %s", original_url)
        return

    # Build tokenized URL.
    tokenized_url = original_url.replace("https://", f"https://{token}:x-oauth-basic@")

    try:
        # Temporarily update the remote URL with the token.
        subprocess.run(["git", "remote", "set-url", "origin", tokenized_url], check=True)
        logging.info("Remote URL updated to tokenized URL.")

        # Push commits.
        subprocess.run(["git", "push", "origin"], check=True)
        logging.info("Pushed commits successfully.")

        # Push the specific tag if provided.
        if tag_name:
            subprocess.run(["git", "push", "origin", tag_name], check=True)
            logging.info("Pushed tag: %s", tag_name)
    except subprocess.CalledProcessError as e:
        logging.error("Error pushing changes: %s", e)
    finally:
        # Reset the remote URL back to its original value.
        subprocess.run(["git", "remote", "set-url", "origin", original_url], check=True)
        logging.info("Reset remote URL to original: %s", original_url)
