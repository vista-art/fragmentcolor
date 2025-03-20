#!/usr/bin/env python3

import re
import sys


def bump_version(file_path, bump_type):
    """Bumbs fragmentcolor version

    Args:
        file_path (_type_): file to bump version in
        bump_type (_type_): type of bump to perform (major, minor, patch)
    """
    with open(file_path, 'r', encoding='utf-8') as file:
        content = file.read()

    version_pattern = (
        r'name = "fragmentcolor"\nversion = "([0-9]+)\.([0-9]+)\.([0-9]+)"'
    )
    match = re.search(version_pattern, content)

    if not match:
        print(f"Version pattern not found in {file_path}")
        return

    major, minor, patch = map(int, match.groups())

    # bumps major version
    if bump_type == 'M':
        major += 1
        minor = 0
        patch = 0
    # bumps minor version
    elif bump_type == 'm':
        minor += 1
        patch = 0
    # bumps patch version
    elif bump_type == 'P':
        patch += 1

    # decreases major version
    elif bump_type == 'D':
        if major > 0:
            major -= 1
        minor = 0
        patch = 0
    # decreases minor version
    elif bump_type == 'd':
        if minor > 0:
            minor -= 1
        patch = 0
    # decreases patch version
    elif bump_type == 'p':
        if patch > 0:
            patch -= 1

    # bumps the patch version by default (most common use case)
    else:
        patch += 1

    new_version = f'{major}.{minor}.{patch}'
    new_content = re.sub(
        version_pattern,
        f'name = "fragmentcolor"\nversion = "{new_version}"',
        content
    )

    with open(file_path, 'w', encoding='utf-8') as file:
        file.write(new_content)

    print(f"Bumped version in {file_path} to {new_version}")


if __name__ == "__main__":
    bump = sys.argv[1] if len(sys.argv) > 1 else ''
    bump_version('Cargo.toml', bump)
    bump_version('pyproject.toml', bump)
