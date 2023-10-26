# tag

## Description

Manage workspace tags.

## Usage

```bash
tfct tag [command] [options]
```

## Global Options

| Short | Long                                | Description                                                    |
|-------|-------------------------------------|----------------------------------------------------------------|
| `-w`  | `--workspace-name <WORKSPACE_NAME>` | The name of the workspace to add the tag to.                   |
| `-i`  | `--workspace-id <WORKSPACE_ID>`     | The id of the workspace to add the tag to.                     |
| `-f`  | `--workspace-file <WORKSPACE_FILE>` | The file containing a list of workspace names or IDs.          |
| `-a`  | `--auto-discover-workspaces`        | Automatically discover workspaces given the specified filters. |

## Subcommands

| Name                    | Description                   |
|-------------------------|-------------------------------|
| [`add`](./add.md)       | Add tags to a workspace.      |
| [`remove`](./remove.md) | Remove tags from a workspace. |
| [`list`](./list.md)     | List tags for a workspace.    |
| `help`                  | Prints help information.      |
