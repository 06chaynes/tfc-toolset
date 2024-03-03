# update

## Description

Update a workspace.

## Usage

```bash
tfct workspace update [options]
```

## Options

| Short | Long                                                | Description                                                                                                                                    |
| ----- | --------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `-w`  | `--workspace-name <WORKSPACE_NAME>`                 | The name of the workspace to update.                                                                                                           |
| `-i`  | `--workspace-id <WORKSPACE_ID>`                     | The ID of the workspace to update.                                                                                                             |
|       | `--name <NAME>`                                     | The new name of the workspace.                                                                                                                 |
|       | `--description <DESC>`                              | A description for the workspace.                                                                                                               |
|       | `--file-triggers-enabled <FILE_TRIGGERS_ENABLED>`   | Whether to filter runs based on the changed files in a VCS push [possible values: true, false]                                                 |
|       | `--trigger-prefixes <TRIGGER_PREFIXES>`             | A list of trigger prefixes that describe the paths Terraform Cloud monitors for changes, in addition to the working directory                  |
|       | `--vcs-branch <VCS_BRANCH>`                         | The repository branch that Terraform will execute from                                                                                         |
|       | `--vcs-identifier <VCS_IDENTIFIER>`                 | A reference to your VCS repository in the format :org/:repo where :org and :repo refer to the organization and repository in your VCS provider |
|       | `--vcs-ingress-submodules <VCS_INGRESS_SUBMODULES>` | Whether to fetch submodules for a VCS repository [possible values: true, false]                                                                |
|       | `--vcs-oauth-token-id <VCS_OAUTH_TOKEN_ID>`         | The VCS Connection (OAuth Connection + Token) to use                                                                                           |
|       | `--vcs-tags-regex <VCS_TAGS_REGEX>`                 | A regular expression used to match Git tags                                                                                                    |
|       | `--terraform-version <TERRAFORM_VERSION>`           | The version of Terraform to use for this workspace                                                                                             |
|       | `--execution-mode <EXECUTION_MODE>`                 | Which execution mode to use for the workspace. Valid values are remote, local, and agent                                                       |
|       | `--auto-apply <AUTO_APPLY>`                         | Whether to automatically apply changes when a Terraform plan is successful [possible values: true, false]                                      |
|       | `--speculative-enabled <SPECULATIVE_ENABLED>`       | Whether this workspace allows automatic speculative plans [possible values: true, false]                                                       |
|       | `--source-url <SOURCE_URL>`                         | A URL identifying the application or client creating this workspace                                                                            |
|       | `--source-name <SOURCE_NAME>`                       | A friendly name for the application or client creating this workspace [default: tfc-toolset]                                                   |
|       | `--queue-all-runs <QUEUE_ALL_RUNS>`                 | Whether runs should be queued immediately after workspace creation [possible values: true, false]                                              |
|       | `--allow-destroy-plan <ALLOW_DESTROY_PLAN>`         | Whether destroy plans can be queued on this workspace [possible values: true, false]                                                           |
|       | `--auto-destroy-at <AUTO_DESTROY_AT>`               | Timestamp (in RFC3339 format) when the next scheduled destroy run will occur                                                                   |
|       | `--assessments-enabled <ASSESSMENTS_ENABLED>`       | Whether or not Terraform Cloud performs health assessments for the workspace [possible values: true, false]                                    |
|       | `--global-remote-state <GLOBAL_REMOTE_STATE>`       | Whether the workspace should allow all workspaces in the organization to access its state data during runs [possible values: true, false]      |
|       | `--tags-regex <TAGS_REGEX>`                         | A regular expression used to match Git tags                                                                                                    |
|       | `--trigger-patterns <TRIGGER_PATTERNS>`             | A list of glob patterns that describe the files Terraform Cloud monitors for changes                                                           |
|       | `--working-directory <WORKING_DIRECTORY>`           | A relative path that Terraform will execute within                                                                                             |
|       | `--agent-pool-id <AGENT_POOL_ID>`                   | The ID of the agent pool belonging to the workspace's organization                                                                             |

## Examples

### Create a workspace

```bash
tfct workspace update --workspace-name "my-workspace" --description "My New Workspace description"
```
