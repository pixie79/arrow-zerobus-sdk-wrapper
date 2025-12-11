#!/usr/bin/env python3
"""
Fetch all comments from a GitHub Pull Request.

This script retrieves all types of comments from a PR:
- Issue comments (general PR comments)
- Review comments (comments on code)
- Review body comments (top-level review comments)

Usage:
    python fetch_pr_comments.py <owner> <repo> <pr_number> [--token TOKEN] [--status STATUS]
    python fetch_pr_comments.py <pr_url> [--token TOKEN] [--status STATUS]

Examples:
    python fetch_pr_comments.py pixie79 otlp-rust-service 123
    python fetch_pr_comments.py https://github.com/pixie79/otlp-rust-service/pull/123
    python fetch_pr_comments.py pixie79 otlp-rust-service 123 --token ghp_xxxxx
    python fetch_pr_comments.py pixie79 otlp-rust-service 123 --status open
    python fetch_pr_comments.py pixie79 otlp-rust-service 123 --status all

Environment Variables:
    GITHUB_TOKEN - GitHub personal access token (optional, can use --token instead)
"""

import argparse
import csv
import json as json_lib
import os
import re
import sys
from datetime import datetime
from typing import Dict, List, Optional
from urllib.request import Request, urlopen
from urllib.error import HTTPError, URLError


class GitHubPRComments:
    """Fetch comments from a GitHub Pull Request."""

    def __init__(self, owner: str, repo: str, token: Optional[str] = None):
        """
        Initialize GitHub API client.

        Args:
            owner: Repository owner (username or organization)
            repo: Repository name
            token: GitHub personal access token (optional, uses GITHUB_TOKEN env var if not provided)
        """
        self.owner = owner
        self.repo = repo
        self.token = token or os.getenv("GITHUB_TOKEN")
        self.base_url = "https://api.github.com"
        self.headers = {
            "Accept": "application/vnd.github.v3+json",
            "User-Agent": "PR-Comments-Fetcher",
        }
        if self.token:
            self.headers["Authorization"] = f"token {self.token}"

    def _make_request(
        self, endpoint: str, method: str = "GET", data: Optional[Dict] = None
    ):
        """
        Make a request to GitHub API and handle pagination (for GET) or single request (for POST).

        Args:
            endpoint: API endpoint (relative to base_url)
            method: HTTP method (GET or POST)
            data: Optional data to send in POST request body

        Returns:
            List of all items from paginated response (GET) or single Dict response (POST)
        """
        if method == "POST":
            # POST request - single response, no pagination
            url = f"{self.base_url}{endpoint}"
            req_data = json_lib.dumps(data).encode("utf-8") if data else None
            req = Request(url, data=req_data, headers=self.headers, method="POST")

            try:
                with urlopen(req) as response:
                    status_code = response.getcode()
                    headers = dict(response.headers)

                    if status_code == 401:
                        print(
                            "Error: Authentication failed. Check your GitHub token.",
                            file=sys.stderr,
                        )
                        sys.exit(1)
                    elif status_code == 404:
                        print(
                            "Error: Not found. Check owner/repo/PR number/comment ID.",
                            file=sys.stderr,
                        )
                        sys.exit(1)
                    elif status_code == 403:
                        print(
                            "Error: Rate limit exceeded or access denied.",
                            file=sys.stderr,
                        )
                        if "X-RateLimit-Remaining" in headers:
                            print(
                                f"Rate limit remaining: {headers['X-RateLimit-Remaining']}",
                                file=sys.stderr,
                            )
                        sys.exit(1)
                    elif status_code not in (200, 201):
                        print(f"Error: HTTP {status_code}", file=sys.stderr)
                        error_body = response.read().decode("utf-8")
                        print(f"Response: {error_body}", file=sys.stderr)
                        sys.exit(1)

                    response_data = response.read().decode("utf-8")
                    return json_lib.loads(response_data)
            except HTTPError as e:
                print(f"Error: HTTP {e.code}: {e.reason}", file=sys.stderr)
                error_body = e.read().decode("utf-8") if hasattr(e, "read") else ""
                print(f"Response: {error_body}", file=sys.stderr)
                sys.exit(1)
            except URLError as e:
                print(
                    f"Error: Failed to connect to GitHub API: {e.reason}",
                    file=sys.stderr,
                )
                sys.exit(1)

        # GET request - handle pagination
        all_items = []
        page = 1
        per_page = 100

        while True:
            url = f"{self.base_url}{endpoint}?page={page}&per_page={per_page}"
            req = Request(url, headers=self.headers)

            try:
                with urlopen(req) as response:
                    status_code = response.getcode()
                    headers = dict(response.headers)

                    if status_code == 401:
                        print(
                            "Error: Authentication failed. Check your GitHub token.",
                            file=sys.stderr,
                        )
                        sys.exit(1)
                    elif status_code == 404:
                        print(
                            "Error: Not found. Check owner/repo/PR number.",
                            file=sys.stderr,
                        )
                        sys.exit(1)
                    elif status_code == 403:
                        print(
                            "Error: Rate limit exceeded or access denied.",
                            file=sys.stderr,
                        )
                        if "X-RateLimit-Remaining" in headers:
                            print(
                                f"Rate limit remaining: {headers['X-RateLimit-Remaining']}",
                                file=sys.stderr,
                            )
                        sys.exit(1)
                    elif status_code != 200:
                        print(f"Error: HTTP {status_code}", file=sys.stderr)
                        sys.exit(1)

                    data = response.read().decode("utf-8")
                    items = json_lib.loads(data)

                    if not items:
                        break

                    all_items.extend(items)

                    # Check if there are more pages
                    if len(items) < per_page:
                        break

                    page += 1
            except HTTPError as e:
                print(f"Error: HTTP {e.code} - {e.reason}", file=sys.stderr)
                sys.exit(1)
            except URLError as e:
                print(f"Error: {e.reason}", file=sys.stderr)
                sys.exit(1)

        return all_items

    def get_pr_info(self, pr_number: int) -> Dict:
        """
        Get basic PR information.

        Args:
            pr_number: Pull request number

        Returns:
            PR information dictionary
        """
        endpoint = f"/repos/{self.owner}/{self.repo}/pulls/{pr_number}"
        url = f"{self.base_url}{endpoint}"
        req = Request(url, headers=self.headers)

        try:
            with urlopen(req) as response:
                if response.getcode() != 200:
                    print(f"Error: HTTP {response.getcode()}", file=sys.stderr)
                    sys.exit(1)
                data = response.read().decode("utf-8")
                return json_lib.loads(data)
        except HTTPError as e:
            print(f"Error: HTTP {e.code} - {e.reason}", file=sys.stderr)
            sys.exit(1)
        except URLError as e:
            print(f"Error: {e.reason}", file=sys.stderr)
            sys.exit(1)

    def get_issue_comments(self, pr_number: int) -> List[Dict]:
        """
        Get all issue comments (general PR comments).

        Args:
            pr_number: Pull request number

        Returns:
            List of issue comments
        """
        endpoint = f"/repos/{self.owner}/{self.repo}/issues/{pr_number}/comments"
        return self._make_request(endpoint)

    def get_review_comments(self, pr_number: int) -> List[Dict]:
        """
        Get all review comments (comments on code).

        Args:
            pr_number: Pull request number

        Returns:
            List of review comments
        """
        endpoint = f"/repos/{self.owner}/{self.repo}/pulls/{pr_number}/comments"
        comments = self._make_request(endpoint)

        # GitHub API doesn't include resolved status in the comments endpoint
        # We need to fetch conversation status separately
        # For now, we'll fetch all comments and mark them based on conversation status
        # Note: This requires checking conversations which may need additional API calls
        return comments

    def get_conversations(self, pr_number: int) -> List[Dict]:
        """
        Get conversation status for review comments.

        Note: GitHub's REST API doesn't directly expose conversation resolved status.
        This method would need to use GraphQL API or check conversations endpoint.

        Args:
            pr_number: Pull request number

        Returns:
            List of conversation statuses (if available)
        """
        # GitHub REST API doesn't have a direct endpoint for conversation status
        # This would require GraphQL API or checking individual conversation threads
        # For now, return empty list - filtering by resolved status is not available via REST API
        return []

    def get_reviews(self, pr_number: int) -> List[Dict]:
        """
        Get all reviews (including review body comments).

        Args:
            pr_number: Pull request number

        Returns:
            List of reviews
        """
        endpoint = f"/repos/{self.owner}/{self.repo}/pulls/{pr_number}/reviews"
        return self._make_request(endpoint)

    def _get_resolved_comment_ids(self, pr_number: int) -> set:
        """
        Get set of resolved review comment IDs using GraphQL API.

        Args:
            pr_number: Pull request number

        Returns:
            Set of resolved comment IDs (empty set if GraphQL not available or error)
        """
        if not self.token:
            # Can't use GraphQL without token
            print(
                "Warning: No GitHub token provided. Cannot check resolved status. Showing all comments.",
                file=sys.stderr,
            )
            return set()

        try:
            # Use GraphQL API to get resolved comment IDs
            graphql_url = "https://api.github.com/graphql"
            query = """
            query($owner: String!, $repo: String!, $prNumber: Int!) {
              repository(owner: $owner, name: $repo) {
                pullRequest(number: $prNumber) {
                  reviewThreads(first: 100) {
                    nodes {
                      isResolved
                      comments(first: 100) {
                        nodes {
                          id
                          databaseId
                        }
                      }
                    }
                  }
                }
              }
            }
            """

            variables = {"owner": self.owner, "repo": self.repo, "prNumber": pr_number}

            payload = json_lib.dumps({"query": query, "variables": variables}).encode(
                "utf-8"
            )
            # GraphQL API requires Bearer token format
            graphql_headers = {
                "Accept": "application/json",
                "Content-Type": "application/json",
                "User-Agent": "PR-Comments-Fetcher",
                "Authorization": f"Bearer {self.token}",
            }
            req = Request(
                graphql_url, data=payload, headers=graphql_headers, method="POST"
            )

            with urlopen(req) as response:
                if response.getcode() != 200:
                    error_body = response.read().decode("utf-8")
                    print(
                        f"GraphQL API returned status {response.getcode()}: {error_body}",
                        file=sys.stderr,
                    )
                    return set()

                data = json_lib.loads(response.read().decode("utf-8"))
                if "errors" in data:
                    print(
                        f"GraphQL API errors: {json_lib.dumps(data['errors'], indent=2)}",
                        file=sys.stderr,
                    )
                    return set()

                resolved_ids = set()
                threads = (
                    data.get("data", {})
                    .get("repository", {})
                    .get("pullRequest", {})
                    .get("reviewThreads", {})
                    .get("nodes", [])
                )
                total_threads = len(threads)
                resolved_threads = 0

                for thread in threads:
                    is_resolved = thread.get("isResolved", False)
                    if is_resolved:
                        resolved_threads += 1
                        comments = thread.get("comments", {}).get("nodes", [])
                        for comment in comments:
                            # GraphQL returns databaseId which matches the REST API id
                            comment_id = comment.get("databaseId")
                            if comment_id:
                                resolved_ids.add(comment_id)

                if total_threads > 0:
                    print(
                        f"Found {total_threads} review threads, {resolved_threads} resolved, {len(resolved_ids)} resolved comments",
                        file=sys.stderr,
                    )
                else:
                    print(
                        "Warning: No review threads found via GraphQL. Showing all comments.",
                        file=sys.stderr,
                    )

                return resolved_ids
        except Exception as e:
            # If GraphQL fails, return empty set (show all comments)
            print(
                f"Warning: Could not fetch resolved status via GraphQL: {e}",
                file=sys.stderr,
            )
            return set()

    def fetch_all_comments(self, pr_number: int, status: str = "open") -> Dict:
        """
        Fetch all comments from a PR.

        Args:
            pr_number: Pull request number
            status: Filter by comment status - "open" (default, unresolved), "resolved", or "all"

        Returns:
            Dictionary containing PR info and all types of comments (filtered by status)
        """
        print(f"Fetching comments for PR #{pr_number}...", file=sys.stderr)

        pr_info = self.get_pr_info(pr_number)
        issue_comments = self.get_issue_comments(pr_number)
        review_comments = self.get_review_comments(pr_number)
        reviews = self.get_reviews(pr_number)

        # Filter review comments by status
        # GitHub REST API doesn't provide resolved status directly
        # We need to use GraphQL API to check conversation resolved status
        if status != "all":
            # Get conversation status using GraphQL API
            resolved_comment_ids = self._get_resolved_comment_ids(pr_number)

            filtered_review_comments = []
            for comment in review_comments:
                comment_id = comment.get("id")
                is_resolved = (
                    comment_id in resolved_comment_ids
                    if resolved_comment_ids
                    else False
                )

                if status == "open" and not is_resolved:
                    filtered_review_comments.append(comment)
                elif status == "resolved" and is_resolved:
                    filtered_review_comments.append(comment)

            review_comments = filtered_review_comments
            if status == "open":
                print(
                    "Filtering to show only open (unresolved) comments...",
                    file=sys.stderr,
                )
            elif status == "resolved":
                print("Filtering to show only resolved comments...", file=sys.stderr)

        return {
            "pr_info": pr_info,
            "issue_comments": issue_comments,
            "review_comments": review_comments,
            "reviews": reviews,
        }

    def reply_to_comment(
        self, pr_number: int, comment_id: int, comment_type: str, reply_body: str
    ) -> Dict:
        """
        Reply to a comment on a PR.

        Args:
            pr_number: Pull request number
            comment_id: ID of the comment to reply to
            comment_type: Type of comment ("issue", "review_comment", or "review")
            reply_body: Text of the reply

        Returns:
            Dictionary containing the created reply comment
        """
        if not self.token:
            print(
                "Error: GitHub token is required for posting replies", file=sys.stderr
            )
            sys.exit(1)

        if comment_type == "review_comment":
            # Reply to a review comment (code comment)
            endpoint = f"/repos/{self.owner}/{self.repo}/pulls/{pr_number}/comments/{comment_id}/replies"
            data = {"body": reply_body}
            result = self._make_request(endpoint, method="POST", data=data)
            return result
        elif comment_type == "issue":
            # Reply to an issue comment (general PR comment)
            # Note: GitHub API uses issue_number for PR comments
            endpoint = f"/repos/{self.owner}/{self.repo}/issues/{pr_number}/comments"
            data = {"body": reply_body, "in_reply_to": comment_id}
            result = self._make_request(endpoint, method="POST", data=data)
            return result
        elif comment_type == "review":
            # Reviews can't be directly replied to via API
            # Instead, we can post a general issue comment
            print(
                "Warning: Reviews cannot be directly replied to. Posting as issue comment instead.",
                file=sys.stderr,
            )
            endpoint = f"/repos/{self.owner}/{self.repo}/issues/{pr_number}/comments"
            data = {"body": reply_body}
            result = self._make_request(endpoint, method="POST", data=data)
            return result
        else:
            print(f"Error: Unknown comment type: {comment_type}", file=sys.stderr)
            sys.exit(1)

    @staticmethod
    def format_comment(comment: Dict, comment_type: str = "comment") -> str:
        """
        Format a comment for display.

        Args:
            comment: Comment dictionary from API
            comment_type: Type of comment (issue, review, review_comment)

        Returns:
            Formatted string
        """
        user = comment.get("user", {})
        user_login = (
            user.get("login", "Unknown") if isinstance(user, dict) else str(user)
        )
        created_at = comment.get("created_at", "")
        body = comment.get("body", "")
        # Handle None body
        if body is None:
            body = ""

        # Parse and format date
        if created_at:
            try:
                dt = datetime.fromisoformat(created_at.replace("Z", "+00:00"))
                date_str = dt.strftime("%Y-%m-%d %H:%M:%S UTC")
            except ValueError:
                date_str = created_at
        else:
            date_str = "Unknown date"

        # Format based on comment type
        if comment_type == "review_comment":
            path = comment.get("path", "Unknown file")
            line = comment.get("line", "?")
            original_line = comment.get("original_line")
            start_line = comment.get("start_line")
            diff_hunk = comment.get("diff_hunk", "")
            in_reply_to_id = comment.get("in_reply_to_id")
            pull_request_review_id = comment.get("pull_request_review_id")

            result = f"[Review Comment] {user_login} on {date_str}\n"
            result += f"File: {path}"
            if line:
                result += f" (line {line})"
            if original_line and original_line != line:
                result += f" (original line {original_line})"
            if start_line:
                result += f" (start line {start_line})"
            result += "\n"

            if in_reply_to_id:
                result += f"Reply to comment ID: {in_reply_to_id}\n"
            if pull_request_review_id:
                result += f"Review ID: {pull_request_review_id}\n"

            if diff_hunk:
                result += f"\nCode Context:\n{diff_hunk}\n"

            if body:
                result += f"\nComment:\n{body}\n"
            else:
                result += "\n(No comment text)\n"
        elif comment_type == "review":
            state = comment.get("state", "unknown")
            result = f"[Review] {user_login} ({state}) on {date_str}\n"
            if body:
                result += f"\nReview Body:\n{body}\n"
            else:
                result += "\n(No review body)\n"
        else:
            result = f"[Comment] {user_login} on {date_str}\n"
            if body:
                result += f"\n{body}\n"
            else:
                result += "\n(No comment text)\n"

        return result + "\n" + "-" * 80 + "\n"

    def print_csv_view(self, data: Dict, output_file: Optional[str] = None):
        """
        Print comments in CSV format.

        Args:
            data: Dictionary containing PR info and comments
            output_file: Optional file path to write CSV to (default: stdout)
        """

        pr_info = data["pr_info"]
        issue_comments = data["issue_comments"]
        review_comments = data["review_comments"]
        reviews = data["reviews"]

        if output_file:
            output = open(output_file, "w", newline="", encoding="utf-8")
        else:
            output = sys.stdout

        try:
            writer = csv.writer(output)

            # Write PR info header
            writer.writerow(
                ["PR Number", "PR Title", "Author", "State", "Created At", "URL"]
            )
            writer.writerow(
                [
                    pr_info["number"],
                    pr_info["title"],
                    pr_info["user"]["login"]
                    if isinstance(pr_info["user"], dict)
                    else "Unknown",
                    pr_info["state"],
                    pr_info["created_at"],
                    pr_info["html_url"],
                ]
            )
            writer.writerow([])  # Empty row

            # Write issue comments
            if issue_comments:
                writer.writerow(
                    ["Comment Type", "ID", "Author", "Created At", "Body", "URL"]
                )
                for comment in issue_comments:
                    writer.writerow(
                        [
                            "issue_comment",
                            comment.get("id", ""),
                            comment.get("user", {}).get("login", "Unknown")
                            if isinstance(comment.get("user"), dict)
                            else "Unknown",
                            comment.get("created_at", ""),
                            comment.get("body", "")
                            .replace("\n", " ")
                            .replace("\r", ""),
                            comment.get("html_url", ""),
                        ]
                    )
                writer.writerow([])  # Empty row

            # Write review comments
            if review_comments:
                writer.writerow(
                    [
                        "Comment Type",
                        "ID",
                        "Author",
                        "Created At",
                        "File",
                        "Line",
                        "Body",
                        "URL",
                    ]
                )
                for comment in review_comments:
                    writer.writerow(
                        [
                            "review_comment",
                            comment.get("id", ""),
                            comment.get("user", {}).get("login", "Unknown")
                            if isinstance(comment.get("user"), dict)
                            else "Unknown",
                            comment.get("created_at", ""),
                            comment.get("path", ""),
                            comment.get("line", ""),
                            comment.get("body", "")
                            .replace("\n", " ")
                            .replace("\r", ""),
                            comment.get("html_url", ""),
                        ]
                    )
                writer.writerow([])  # Empty row

            # Write reviews
            if reviews:
                writer.writerow(
                    [
                        "Comment Type",
                        "ID",
                        "Author",
                        "State",
                        "Created At",
                        "Body",
                        "URL",
                    ]
                )
                for review in reviews:
                    writer.writerow(
                        [
                            "review",
                            review.get("id", ""),
                            review.get("user", {}).get("login", "Unknown")
                            if isinstance(review.get("user"), dict)
                            else "Unknown",
                            review.get("state", ""),
                            review.get("submitted_at") or review.get("created_at", ""),
                            review.get("body", "").replace("\n", " ").replace("\r", ""),
                            review.get("html_url", ""),
                        ]
                    )
        finally:
            if output_file and output:
                output.close()
                print(f"CSV exported to {output_file}", file=sys.stderr)

    def print_table_view(self, data: Dict):
        """
        Print comments in a formatted table view.

        Args:
            data: Dictionary containing PR info and comments
        """
        pr_info = data["pr_info"]
        issue_comments = data["issue_comments"]
        review_comments = data["review_comments"]
        # reviews = data.get("reviews", [])  # Unused, kept for future use

        # PR Header
        print(f"\n{'=' * 100}")
        print(f"PR #{pr_info['number']}: {pr_info['title']}")
        print(
            f"Author: {pr_info['user']['login']} | State: {pr_info['state']} | Created: {pr_info['created_at']}"
        )
        print(f"URL: {pr_info['html_url']}")
        print(f"{'=' * 100}\n")

        # Helper function to truncate text
        def truncate(text: str, max_len: int = 60) -> str:
            if not text:
                return ""
            text = text.replace("\n", " ").replace("\r", "")
            if len(text) > max_len:
                return text[: max_len - 3] + "..."
            return text

        # Issue Comments Table
        if issue_comments:
            print(f"\n{'─' * 100}")
            print(f"ISSUE COMMENTS ({len(issue_comments)} total)")
            print(f"{'─' * 100}")
            print(f"{'ID':<12} {'Author':<20} {'Date':<20} {'Comment':<48}")
            print(f"{'─' * 100}")
            for comment in issue_comments:
                comment_id = str(comment.get("id", "N/A"))
                user = (
                    comment.get("user", {}).get("login", "Unknown")
                    if isinstance(comment.get("user"), dict)
                    else "Unknown"
                )
                created = (
                    comment.get("created_at", "")[:19]
                    if comment.get("created_at")
                    else "Unknown"
                )
                body = truncate(comment.get("body", ""), 48)
                print(f"{comment_id:<12} {user:<20} {created:<20} {body:<48}")
            print(f"{'─' * 100}\n")

        # Review Comments Table
        if review_comments:
            print(f"\n{'─' * 100}")
            print(f"REVIEW COMMENTS - Code Comments ({len(review_comments)} total)")
            print(f"{'─' * 100}")
            print(f"{'ID':<12} {'Author':<20} {'File':<35} {'Line':<8} {'Comment':<25}")
            print(f"{'─' * 100}")
            for comment in review_comments:
                comment_id = str(comment.get("id", "N/A"))
                user = (
                    comment.get("user", {}).get("login", "Unknown")
                    if isinstance(comment.get("user"), dict)
                    else "Unknown"
                )
                path = comment.get("path", "Unknown")
                if len(path) > 33:
                    path = "..." + path[-30:]
                line = str(comment.get("line", "?"))
                body = truncate(comment.get("body", ""), 25)
                print(f"{comment_id:<12} {user:<20} {path:<35} {line:<8} {body:<25}")
            print(f"{'─' * 100}\n")

        # Summary (reviews not shown in table view - only comments are displayed)
        total = len(issue_comments) + len(review_comments)
        print(f"{'=' * 100}")
        print(
            f"SUMMARY: {len(issue_comments)} Issue Comments | {len(review_comments)} Review Comments | Total: {total}"
        )
        print(f"{'=' * 100}\n")

    def print_all_comments(self, data: Dict):
        """
        Print all comments in a readable format.

        Args:
            data: Dictionary containing PR info and comments
        """
        pr_info = data["pr_info"]
        print(f"\n{'=' * 80}")
        print(f"PR #{pr_info['number']}: {pr_info['title']}")
        print(f"Author: {pr_info['user']['login']}")
        print(f"State: {pr_info['state']}")
        print(f"Created: {pr_info['created_at']}")
        print(f"URL: {pr_info['html_url']}")
        print(f"{'=' * 80}\n")

        # Print issue comments
        issue_comments = data["issue_comments"]
        if issue_comments:
            print(f"\n{'=' * 80}")
            print(f"ISSUE COMMENTS ({len(issue_comments)} total)")
            print(f"{'=' * 80}\n")
            for comment in issue_comments:
                print(self.format_comment(comment, "issue"))

        # Print review comments
        review_comments = data["review_comments"]
        if review_comments:
            print(f"\n{'=' * 80}")
            print(f"REVIEW COMMENTS (Code Comments) ({len(review_comments)} total)")
            print(f"{'=' * 80}\n")
            for comment in review_comments:
                print(self.format_comment(comment, "review_comment"))

        # Print reviews
        reviews = data["reviews"]
        if reviews:
            print(f"\n{'=' * 80}")
            print(f"REVIEWS ({len(reviews)} total)")
            print(f"{'=' * 80}\n")
            for review in reviews:
                print(self.format_comment(review, "review"))

        # Summary
        total = len(issue_comments) + len(review_comments) + len(reviews)
        print(f"\n{'=' * 80}")
        print("SUMMARY")
        print(f"{'=' * 80}")
        print(f"Issue Comments: {len(issue_comments)}")
        print(f"Review Comments: {len(review_comments)}")
        print(f"Reviews: {len(reviews)}")
        print(f"Total: {total}")
        print(f"{'=' * 80}\n")


def parse_pr_url(url: str) -> tuple:
    """
    Parse a GitHub PR URL to extract owner, repo, and PR number.

    Args:
        url: GitHub PR URL

    Returns:
        Tuple of (owner, repo, pr_number)
    """
    # Pattern: https://github.com/owner/repo/pull/123
    pattern = r"github\.com/([^/]+)/([^/]+)/pull/(\d+)"
    match = re.search(pattern, url)
    if match:
        return match.groups()[0], match.groups()[1], int(match.groups()[2])
    raise ValueError(f"Invalid PR URL format: {url}")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Fetch all comments from a GitHub Pull Request",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument(
        "owner_or_url",
        nargs="?",
        help="Repository owner (username/org) or full PR URL",
    )
    parser.add_argument(
        "repo",
        nargs="?",
        help="Repository name (required if owner is provided, not if URL is provided)",
    )
    parser.add_argument(
        "pr_number",
        nargs="?",
        type=int,
        help="Pull request number (required if owner/repo provided, not if URL is provided)",
    )
    parser.add_argument(
        "--token",
        help="GitHub personal access token (or set GITHUB_TOKEN env var)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output as JSON (useful for AI/automation)",
    )
    parser.add_argument(
        "--table",
        action="store_true",
        help="Output as formatted table (default: detailed text format)",
    )
    parser.add_argument(
        "--csv",
        nargs="?",
        const="",
        metavar="FILE",
        help="Output as CSV (optionally specify output file, default: stdout)",
    )
    parser.add_argument(
        "--detail",
        type=int,
        metavar="COMMENT_ID",
        help="Get details for a specific comment by ID",
    )
    parser.add_argument(
        "--comment-type",
        choices=["issue", "review_comment", "review"],
        help="Comment type for --detail or --reply (required with --detail or --reply)",
    )
    parser.add_argument(
        "--reply",
        type=int,
        metavar="COMMENT_ID",
        help="Reply to a specific comment by ID",
    )
    parser.add_argument(
        "--reply-body",
        help="Reply body text (required with --reply)",
    )
    parser.add_argument(
        "--reply-body-file",
        help="File containing reply body text (alternative to --reply-body)",
    )
    parser.add_argument(
        "--status",
        choices=["open", "resolved", "all"],
        default="open",
        help="Filter comments by status: 'open' (default, unresolved comments), 'resolved', or 'all' (all comments). Note: Requires --token for status filtering to work.",
    )

    args = parser.parse_args()

    # Parse arguments - support both URL and owner/repo/number formats
    if not args.owner_or_url:
        parser.print_help()
        sys.exit(1)

    # Check if it's a URL
    if args.owner_or_url.startswith("http"):
        try:
            owner, repo, pr_number = parse_pr_url(args.owner_or_url)
        except ValueError as e:
            print(f"Error: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        if not args.repo or not args.pr_number:
            print(
                "Error: Must provide both repo and pr_number when using owner format",
                file=sys.stderr,
            )
            parser.print_help()
            sys.exit(1)
        owner = args.owner_or_url
        repo = args.repo
        pr_number = args.pr_number

    # Initialize client
    client = GitHubPRComments(owner, repo, token=args.token)

    # Handle reply to comment
    if args.reply:
        if not args.comment_type:
            print("Error: --comment-type is required with --reply", file=sys.stderr)
            sys.exit(1)
        if not args.reply_body and not args.reply_body_file:
            print(
                "Error: --reply-body or --reply-body-file is required with --reply",
                file=sys.stderr,
            )
            sys.exit(1)
        if not args.token:
            print("Error: --token is required for --reply", file=sys.stderr)
            sys.exit(1)

        reply_body = args.reply_body
        if args.reply_body_file:
            with open(args.reply_body_file, "r", encoding="utf-8") as f:
                reply_body = f.read()

        result = client.reply_to_comment(
            pr_number, args.reply, args.comment_type, reply_body
        )
        print("Reply posted successfully!")
        print(f"Comment ID: {result.get('id')}")
        print(f"URL: {result.get('html_url')}")
        sys.exit(0)

    # Handle get single comment detail
    if args.detail:
        if not args.comment_type:
            print("Error: --comment-type is required with --detail", file=sys.stderr)
            sys.exit(1)

        comment = client.get_comment_by_id(args.detail, args.comment_type)
        if not comment:
            print(f"Error: Comment {args.detail} not found", file=sys.stderr)
            sys.exit(1)

        if args.json:
            print(json_lib.dumps(comment, indent=2))
        else:
            # Format single comment
            comment_type_map = {
                "issue": "issue",
                "review_comment": "review_comment",
                "review": "review",
            }
            print(
                client.format_comment(
                    comment, comment_type_map.get(args.comment_type, "issue")
                )
            )
        sys.exit(0)

    # Fetch all comments (with status filter)
    data = client.fetch_all_comments(pr_number, status=args.status)

    # Output results
    if args.json:
        # Enhanced JSON output for AI consumption
        json_output = {
            "pr": {
                "number": data["pr_info"]["number"],
                "title": data["pr_info"]["title"],
                "author": data["pr_info"]["user"]["login"]
                if isinstance(data["pr_info"]["user"], dict)
                else "Unknown",
                "state": data["pr_info"]["state"],
                "created_at": data["pr_info"]["created_at"],
                "url": data["pr_info"]["html_url"],
            },
            "comments": {
                "issue_comments": [
                    {
                        "id": c.get("id"),
                        "user": c.get("user", {}).get("login", "Unknown")
                        if isinstance(c.get("user"), dict)
                        else "Unknown",
                        "created_at": c.get("created_at"),
                        "body": c.get("body", ""),
                        "url": c.get("html_url"),
                    }
                    for c in data["issue_comments"]
                ],
                "review_comments": [
                    {
                        "id": c.get("id"),
                        "user": c.get("user", {}).get("login", "Unknown")
                        if isinstance(c.get("user"), dict)
                        else "Unknown",
                        "created_at": c.get("created_at"),
                        "path": c.get("path"),
                        "line": c.get("line"),
                        "body": c.get("body", ""),
                        "diff_hunk": c.get("diff_hunk"),
                        "url": c.get("html_url"),
                    }
                    for c in data["review_comments"]
                ],
                "reviews": [
                    {
                        "id": r.get("id"),
                        "user": r.get("user", {}).get("login", "Unknown")
                        if isinstance(r.get("user"), dict)
                        else "Unknown",
                        "state": r.get("state"),
                        "created_at": r.get("created_at"),
                        "body": r.get("body", ""),
                        "url": r.get("html_url"),
                    }
                    for r in data["reviews"]
                ],
            },
            "summary": {
                "total_issue_comments": len(data["issue_comments"]),
                "total_review_comments": len(data["review_comments"]),
                "total_reviews": len(data["reviews"]),
                "total_comments": len(data["issue_comments"])
                + len(data["review_comments"])
                + len(data["reviews"]),
            },
        }
        print(json_lib.dumps(json_output, indent=2))
    elif args.csv is not None:
        output_file = args.csv if args.csv else None
        client.print_csv_view(data, output_file)
    elif args.table:
        client.print_table_view(data)
    else:
        client.print_all_comments(data)


if __name__ == "__main__":
    main()
