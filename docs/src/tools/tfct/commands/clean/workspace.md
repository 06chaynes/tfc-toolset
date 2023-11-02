# workspace

## Description

Run workspace clean operations

## Usage

```bash
tfct clean workspace [options]
```

## Options

| Short | Long                                             | Description                                                                                                           |
|-------|--------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------|
| `-w`  | `--workspace-name <WORKSPACE_NAME>`              | The name of the workspace to create the run on.                                                                       |
| `-i`  | `--workspace-id <WORKSPACE_ID>`                  | The id of the workspace to create the run on.                                                                         |
| `-f`  | `--workspace-file <WORKSPACE_FILE>`              | The file containing a list of workspace names or IDs.                                                                 |
| `-a`  | `--auto-discover-workspaces`                     | Automatically discover workspaces given the specified filters.                                                        |
| `-d`  | `--dry-run <DRY_RUN>`                            | Do not perform any clean operations, only detect issues to be cleaned [default: true] [possible values: true, false]. |
| `-u`  | `--unlisted-variables <UNLISTED_VARIABLES>`      | Detect/Remove unlisted variables [default: true] [possible values: true, false].                                      |
| `-m`  | `--missing-repositories <MISSING_REPOSITORIES>`  | Detect missing vcs repositories [default: false] [possible values: true, false].                                      |
|       | `--git-dir <GIT_DIR>`                            | Override the location to which git repositories are cloned.                                                           |
|       | `--message <MESSAGE>`                            | A message to include with the run [default: "Run created by tfc-toolset"].                                            |
|       | `--target-addrs <TARGET_ADDRS>`                  | A list of resource addresses to target for the run.                                                                   |
|       | `--replace-addrs <REPLACE_ADDRS>`                | A list of resource addresses to replace for the run.                                                                  |
|       | `--terraform-version <TERRAFORM_VERSION>`        | The version of Terraform to use for this run, overriding the value from settings.                                     |

## Examples

### Detect issues in a workspace

```bash
tfct clean workspace --workspace-name "my-workspace"
```

### Detect issues in a workspace and save the results to a file

```bash
tfct clean workspace --workspace-name "my-workspace" --save-output --output "clean-results.json"
```

### Detect and attempt to fix issues in a workspace

```bash
tfct clean workspace --workspace-name "my-workspace" --dry-run false
```