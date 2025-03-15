"""
This module is responsible for configuring the Git repository for our version
management tasks. It includes functions to:
  - Mark the repository as a safe directory (avoiding dubious ownership issues).
  - Set a default Git identity (user.name and user.email) if not already configured.

These settings ensure that all Git operations (commits, pushes, etc.) performed
by our automation script run smoothly in environments like containers.
"""

import logging
from git import Repo

def configure_safe_directory(repo_path):
    """
    Configure the repository as a safe directory.

    In certain environments (e.g. Docker containers), Git might refuse to operate
    because the repository ownership does not match the current user. This function
    marks the repository as safe to avoid such errors.

    Parameters:
        repo_path (str): Absolute path to the repository.

    Returns:
        None
    """
    try:
        with Repo(repo_path).config_writer(config_level='global') as cw:
            cw.set_value("safe", "directory", repo_path)
        logging.info("Safe directory configured for repository: %s", repo_path)
    except Exception as e:
        logging.error("Failed to configure safe directory for %s: %s", repo_path, e)

def configure_git_identity(repo):
    """
    Ensure that a default Git identity is set for the repository.

    This function checks if the repository has a user.name and user.email set. If not,
    it sets default values. This is required to avoid commit errors, especially in
    automated environments where Git might not auto-detect an identity.

    Parameters:
        repo (git.Repo): GitPython Repo object representing the current repository.

    Returns:
        None
    """
    try:
        config_reader = repo.config_reader()
    except Exception as e:
        logging.warning("Unable to read Git configuration: %s", e)
        config_reader = None

    try:
        with repo.config_writer() as cw:
            # Check and set user.name if necessary
            try:
                name = config_reader.get_value("user", "name")
            except Exception:
                name = None
            if not name:
                cw.set_value("user", "name", "GitHub Actions")
                logging.info("Set default git user.name to 'GitHub Actions'")
            # Check and set user.email if necessary
            try:
                email = config_reader.get_value("user", "email")
            except Exception:
                email = None
            if not email:
                cw.set_value("user", "email", "actions@github.com")
                logging.info("Set default git user.email to 'actions@github.com'")
    except Exception as e:
        logging.error("Failed to configure Git identity: %s", e)
