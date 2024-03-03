# delete

## Description

Delete variables from a workspace.

## Usage

```bash
tfct variable delete [options]
```

## Options

| Short | Long                    | Description                                           |
| ----- | ----------------------- | ----------------------------------------------------- |
| `-k`  | `--var-key <VAR_KEY>`   | The key of the variable to delete from the workspace. |
| `-v`  | `--var-id <VAR_ID>`     | The id of the variable to delete from the workspace.  |
|       | `--var-file <VAR_FILE>` | The file containing variables.                        |

## Examples

### Delete a variable from a workspace

```bash
tfct variable delete --workspace-name "my-workspace" --var-key "SECRET_KEY"
```

```bash
tfct variable delete --workspace--name "my-workspace" --var-id "var-id"
```

### Delete a variable from a workspace using a file

```json
{
  "variables": [
    {
      "var": "SECRET_KEY=mysecretohno:ENV var for the secret:env:false:true"
    },
    {
      "var": "newthing=vale::::true"
    }
  ]
}
```

```bash
tfct variable delete --workspace-id "ws-id" --var-file vars.json
```
