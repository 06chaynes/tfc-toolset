# remove

## Description

Remove a workspace from a variable set.

## Usage

```bash
tfct variable-set remove [options]
```

## Options

| Short | Long                        | Description                 |
| ----- | --------------------------- | --------------------------- |
| `-v`  | `--var-set-id <VAR_SET_ID>` | The ID of the variable set. |

## Examples

### Remove a workspace to a variable set

```bash
tfct variable-set remove --workspace-name "my-workspace" -v "varset-id"
```
