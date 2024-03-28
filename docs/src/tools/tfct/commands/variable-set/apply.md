# apply

## Description

Apply a workspace to a variable set.

## Usage

```bash
tfct variable-set apply [options]
```

## Options

| Short | Long                        | Description                 |
| ----- | --------------------------- | --------------------------- |
| `-v`  | `--var-set-id <VAR_SET_ID>` | The ID of the variable set. |

## Examples

### Apply a workspace to a variable set

```bash
tfct variable-set apply --workspace-name "my-workspace" -v "varset-id"
```
