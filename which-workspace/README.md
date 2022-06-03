# tfc-which-workspace

## What is this?

This tool is for searching for workspaces in Terraform Cloud. It first attempts to pull a list of workspaces for the specified organization, it can optionally accept a name parameter to filter the initial list. After the list of workspaces has been retrieved the tool can then attempt to further filter the list based on the provided query parameters.

If one or more filters have been provided the tool will attempt to work through the passed filter rules. For each workspace in the list an additional call will be made to pull the variables for that workspace. After that data has been gathered it will then run through the filter logic. See [Filtering](../FILTERING.md) for more details

After the filter logic has completed the resulting dataset will be outputted both to the terminal and to a file. By default the file name will be `result.json` and it will be placed in the working directory in which the tool was run.

Settings can be provided by either providing a `settings.toml` file or by passing ENV variables along before the command. Caching is leveraged on all remote calls (following http caching rules) and will create a directory in the working directory in which the tool was run named `http-cacache` where the cache files will reside.

## Why tho?

Because I didn't want to do it manually and I felt like it
