# Shared Profile Catalog

This directory contains portable VoxTypePersonas profile definitions that can be reviewed, edited, and contributed through pull requests.

Each profile is a Markdown file with YAML front matter. The front matter holds machine-readable, non-secret metadata; the Markdown body is the system prompt. Files must never contain API keys, Secret Service references, dictated text, or personal configuration.

Imported profiles are drafts by default. A draft can be viewed, edited, and duplicated, but cannot become active until it has a compatible provider, an explicitly selected model, and a verified local Secret Service key when the provider requires one.

Before accepting an import, VoxTypePersonas must validate the complete file: its schema version, required and typed metadata, unique profile ID, supported provider metadata, and the absence of prohibited private data. Invalid or unsafe files must be rejected atomically, without creating a partial local profile.

Contributions should include compatibility notes and should describe how the prompt was evaluated. A profile must not claim model-independent quality or guaranteed results.

## CLI workflow

Validate one or more files without reading or creating local configuration:

```text
voxtype-personas profiles validate <file>...
```

Import one or more validated files as local Draft profiles:

```text
voxtype-personas profiles import <file>...
```

The engine validates every requested file and all profile IDs before it writes local configuration. A successful import is atomic. It creates a local Draft with the portable prompt, limits, and output policy; provider and model must be configured locally before the profile can become Ready. An invalid file or local ID collision leaves the existing configuration unchanged.

Export one non-Raw local profile to a new portable file:

```text
voxtype-personas profiles export <id> <file>
```

Export always writes a Draft and never serializes API keys or Secret Service references. It refuses to overwrite an existing file.

## Format

```markdown
---
schema_version: 1
id: example
name: Example profile
status: draft

provider:
  kind: openai_compatible
  endpoint: null
  model: null

compatibility:
  model_families: []
  notes: Tune this prompt for the selected model before activating it.

limits:
  max_input_chars: 20000
  max_output_tokens: 2048
  timeout_ms: 30000

output_policy:
  reject_markdown: true
  reject_obvious_preambles: true
---

System prompt text goes here.
```
