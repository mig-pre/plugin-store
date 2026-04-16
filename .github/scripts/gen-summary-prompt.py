#!/usr/bin/env python3
"""Generate the Claude API prompt for SUMMARY.md."""
import sys, os

name = sys.argv[1]
plugin_dir = sys.argv[2]

yaml_path = os.path.join(plugin_dir, "plugin.yaml")
readme_path = os.path.join(plugin_dir, "README.md")

yaml_content = open(yaml_path).read() if os.path.exists(yaml_path) else ""
readme_content = open(readme_path).read() if os.path.exists(readme_path) else ""
skill_content = ""
if os.path.exists("/tmp/skill_content.txt"):
    skill_content = "".join(open("/tmp/skill_content.txt").readlines()[:500])

prompt = f"""You are generating documentation for plugin "{name}".

Given the SKILL.md, README.md, and plugin.yaml below, generate a SUMMARY.md file.

Output the markdown content directly (no separators needed).

SUMMARY.md format:
# {name}
<one sentence description>
## Highlights
- <feature 1>
- <feature 2>
...up to 8 highlights

=== INPUT ===

plugin.yaml:
{yaml_content}

README.md:
{readme_content}

SKILL.md:
{skill_content}
"""

with open("/tmp/prompt.txt", "w") as f:
    f.write(prompt)
print(f"Prompt written: {len(prompt)} chars")
