# add

## Description

Add tags to a workspace.

## Usage

```bash
tfct tag add [options]
```

## Options

| Short | Long                    | Description                                |
| ----- | ----------------------- | ------------------------------------------ |
| `-n`  | `--name <NAME>`         | The name of the tag to add.                |
|       | `--tag-file <TAG_FILE>` | The file containing a list of tags to add. |

## Examples

### Add a tag to a workspace

```bash
tfct tag add --workspace-name "my-workspace" --name my-tag
```

### Add a tag to a workspace using a file

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
tfct tag add --workspace-id "ws-id" --tag-file tags.json
```
