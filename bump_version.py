#!/usr/bin/env python3

import re
import sys


import json
from pathlib import Path

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

    # Also bump the website package.json top-level version (not dependencies here)
    site_pkg = Path('docs/website/package.json')
    if site_pkg.exists():
        try:
            with site_pkg.open('r', encoding='utf-8') as f:
                pkg = json.load(f)
            old = pkg.get('version')
            pkg['version'] = new_version
            with site_pkg.open('w', encoding='utf-8') as f:
                json.dump(pkg, f, ensure_ascii=False, indent=2)
            print(f"Bumped version in {site_pkg} from {old} to {new_version}")
        except Exception as e:
            print(f"Warning: failed updating {site_pkg}: {e}")

    readme = Path('README.md')
    if readme.exists():
        try:
            content = readme.read_text(encoding='utf-8')
            content = re.sub(
                r'(fragmentcolor = ")([0-9]+\.[0-9]+\.[0-9]+)(")',
                rf'\g<1>{new_version}\3',
                content,
                count=1,
            )
            readme.write_text(content, encoding='utf-8')
            print(f"Bumped Rust install example in {readme} to {new_version}")
        except Exception as e:
            print(f"Warning: failed updating {readme}: {e}")

    py_reqs = Path('examples/python/requirements.txt')
    if py_reqs.exists():
        try:
            content = py_reqs.read_text(encoding='utf-8')
            content = re.sub(
                r'(fragmentcolor>=)([0-9]+\.[0-9]+\.[0-9]+)',
                rf'\g<1>{new_version}',
                content,
                count=1,
            )
            py_reqs.write_text(content, encoding='utf-8')
            print(f"Bumped Python example requirement in {py_reqs} to {new_version}")
        except Exception as e:
            print(f"Warning: failed updating {py_reqs}: {e}")

    badge = Path('docs/website/src/components/VersionBadge.astro')
    if badge.exists():
        try:
            content = badge.read_text(encoding='utf-8')
            content = re.sub(
                r"(const VERSION = ')([0-9]+\.[0-9]+\.[0-9]+)(';)",
                rf"\g<1>{new_version}\3",
                content,
                count=1,
            )
            badge.write_text(content, encoding='utf-8')
            print(f"Bumped website version badge in {badge} to {new_version}")
        except Exception as e:
            print(f"Warning: failed updating {badge}: {e}")

    return new_version


if __name__ == "__main__":
    bump = sys.argv[1] if len(sys.argv) > 1 else ''
    new_ver = bump_version('Cargo.toml', bump)
    bump_version('pyproject.toml', bump)
    print(f"New version: {new_ver}")
