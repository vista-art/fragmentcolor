#!/usr/bin/env python3

import re
import sys


def bump_version(file_path, bump_type):
    with open(file_path, 'r') as file:
        content = file.read()

    version_pattern = (
        r'name = "fragmentcolor"\nversion = "([0-9]+)\.([0-9]+)\.([0-9]+)"'
    )
    match = re.search(version_pattern, content)

    if not match:
        print(f"Version pattern not found in {file_path}")
        return

    major, minor, patch = map(int, match.groups())

    if bump_type == 'm':
        minor += 1
        patch = 0
    elif bump_type == 'M':
        major += 1
        minor = 0
        patch = 0
    elif bump_type == 'd':
        if minor > 0:
            minor -= 1
        patch = 0
    elif bump_type == 'D':
        if major > 0:
            major -= 1
        minor = 0
        patch = 0
    elif bump_type == 'p':
        if patch > 0:
            patch -= 1
    else:
        patch += 1

    new_version = f'{major}.{minor}.{patch}'
    new_content = re.sub(
        version_pattern,
        f'name = "fragmentcolor"\nversion = "{new_version}"',
        content
    )

    with open(file_path, 'w') as file:
        file.write(new_content)

    print(f"Bumped version in {file_path} to {new_version}")


if __name__ == "__main__":
    bump_type = sys.argv[1] if len(sys.argv) > 1 else ''
    bump_version('Cargo.toml', bump_type)
    bump_version('pyproject.toml', bump_type)
