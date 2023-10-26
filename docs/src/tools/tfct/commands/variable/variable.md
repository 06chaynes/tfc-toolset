# variable

## Description

Manage workspace variables.

## Usage

```bash
tfct variable [command] [options]
```

## Global Options

| Short | Long                                | Description                                                    |
|-------|-------------------------------------|----------------------------------------------------------------|
| `-w`  | `--workspace-name <WORKSPACE_NAME>` | The name of the workspace to add the tag to.                   |
| `-i`  | `--workspace-id <WORKSPACE_ID>`     | The id of the workspace to add the tag to.                     |
| `-f`  | `--workspace-file <WORKSPACE_FILE>` | The file containing a list of workspace names or IDs.          |
| `-a`  | `--auto-discover-workspaces`        | Automatically discover workspaces given the specified filters. |

## Subcommands

| Name                    | Description                        |
|-------------------------|------------------------------------|
| [`create`](./create.md) | Create variables on a workspace.   |
| [`delete`](./delete.md) | Delete variables from a workspace. |
| [`list`](./list.md)     | List variables for a workspace.    |
| `help`                  | Prints help information.           |