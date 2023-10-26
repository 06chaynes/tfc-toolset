# remove

## Description

Remove tags from a workspace.

## Usage

```bash
tfct tag remove [options]
```

## Options

| Short | Long                      | Description                                   |
|-------|---------------------------|-----------------------------------------------|
| `-n`  | `--name <NAME>`           | The name of the tag to remove.                |
|       | `--tag-file <TAG_FILE>`   | The file containing a list of tags to remove. |

## Examples

### Remove a tag to a workspace

```bash
tfct tag remove --workspace-name "my-workspace" --name my-tag
```

### Remove a tag to a workspace using a file

```json
{
  "tags": [
    {
      "name": "it:worked"
    }
  ]
}
```

```bash
tfct tag remove --workspace-id "ws-id" --tag-file tags.json
```