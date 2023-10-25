# tag

## Description

Manage workspace tags.

## Usage

```bash
tfct tag [command]
```

## Global Options

| Short | Long                                | Description                                                    |
|-------|-------------------------------------|----------------------------------------------------------------|
| `-w`  | `--workspace-name <WORKSPACE_NAME>` | The name of the workspace to add the tag to.                   |
| `-i`  | `--workspace-id <WORKSPACE_ID>`     | The id of the workspace to add the tag to.                     |
| `-f`  | `--workspace-file <WORKSPACE_FILE>` | The file containing a list of workspace names or IDs.          |
| `-a`  | `--auto-discover-workspaces`        | Automatically discover workspaces given the specified filters. |

## Subcommands

| Name     | Description                    |
|----------|--------------------------------|
| `add`    | Add a tag to a workspace.      |
| `remove` | Remove a tag from a workspace. |
| `list`   | List tags for a workspace.     |
| `help`   | Prints help information.       |

### add

#### Usage

```bash
tfct tag add [options]
```

#### Options

| Short | Long                      | Description                                                    |
|-------|---------------------------|----------------------------------------------------------------|
| `-n`  | `--name <NAME>`           | The name of the tag to add.                                    |
|       | `--tag-file <TAG_FILE>`   | The file containing a list of tags to add.                     |

#### Examples

```bash
# Add a tag to a workspace
tfct tag add --workspace-name "My Workspace" --name my-tag

# Add a tag to a workspace using a file
tfct tag add --workspace-name "My Workspace" --tag-file tags.json
```
