# Memory Templates

Reference templates for creating `.phi/memory/*.md` files.

The agent uses `read_file` to load templates and `write_file` to persist memories
following the same frontmatter convention.

## Usage

The agent discovers these templates via the system prompt and can read them
to guide memory creation:

```
# LLM workflow:
1. read_file(".phi/templates/memory/project-overview.md")  <- read template
2. write_file(".phi/memory/my-project.md", content)        <- write memory
3. read_file(".phi/memory/MEMORY.md")                      <- read index
4. write_file(".phi/memory/MEMORY.md", updated_index)       <- update index
```

## Templates

| Template | Type | Use Case |
|----------|------|----------|
| `project-overview.md` | project | New project context, goals, constraints |
| `coding-style.md` | feedback | Coding conventions, style preferences |
| `user-preferences.md` | user | User role, expertise, workflow habits |
