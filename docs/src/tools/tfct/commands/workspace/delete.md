# delete

## Description

Delete a workspace.

## Usage

```bash
tfct workspace delete [options]
```

## Options

| Short | Long                                | Description                                                             |
| ----- | ----------------------------------- | ----------------------------------------------------------------------- |
| `-w`  | `--workspace-name <WORKSPACE_NAME>` | The name of the workspace to show.                                      |
| `-i`  | `--workspace-id <WORKSPACE_ID>`     | The id of the workspace to show.                                        |
| `-s`  | `--safe`                            | Delete an existing workspace, but only if it is not managing resources. |

## Examples

### Delete a workspace

```bash
tfct workspace delete --workspace-name "my-workspace"
```
