# variable-set

## Description

Manage variable sets.

## Usage

```bash
tfct variable-set [command] [options]
```

## Global Options

| Short | Long                                | Description                                                    |
|-------|-------------------------------------|----------------------------------------------------------------|
| `-w`  | `--workspace-name <WORKSPACE_NAME>` | The name of the workspace to add the tag to.                   |
| `-i`  | `--workspace-id <WORKSPACE_ID>`     | The id of the workspace to add the tag to.                     |
| `-f`  | `--workspace-file <WORKSPACE_FILE>` | The file containing a list of workspace names or IDs.          |
| `-a`  | `--auto-discover-workspaces`        | Automatically discover workspaces given the specified filters. |

## Subcommands

| Name                    | Description                             |
|-------------------------|-----------------------------------------|
| [`apply`](./apply.md)   | Apply a workspace to a variable set.    |
| [`remove`](./remove.md) | Remove a workspace from a variable set. |
| `help`                  | Prints help information.                |