# tfc-which-workspace

## What is this?

This tool is for searching for workspaces in Terraform Cloud. It first attempts to pull a list of workspaces for the specified organization, it can optionally accept a name parameter to filter the initial list. After the list of workspaces has been retrieved the tool can then attempt to further filter the list based on the provided query parameters.

If one or more filters have been provided the tool will attempt to work through the passed filter rules. For each workspace in the list an additional call will be made to pull the variables for that workspace. After that data has been gathered it will then run through the filter logic.

After the filter logic has completed the resulting dataset will be outputted both to the terminal and to a file. By default the file name will be `result.json` and it will be placed in the working directory in which the tool was run.

Settings can be provided by either providing a `settings.toml` file or by passing ENV variables along before the command. Caching is leveraged on all remote calls (following http caching rules) and will create a directory in the working directory in which the tool was run named `http-cacache` where the cache files will reside.

## Why tho?

Because I didn't want to do it manually and I felt like it

## Query Filters

Filters can be combined as needed. The current run order for filter logic is:

1. Name
2. Tags
3. Variables

### Variable Filters

First let's take a look at an example variable filter setup.

```toml
...
[query] # Required but can be left empty
name = "aws-" # Optional

[[query.variables]] # Optional
key = "mode" # Required
operator = "Contains" # Required
value = "prod" # Required

[[query.variables]] # Optional
key = "status" # Required
operator = "NotEqual" # Required
value = "migrating" # Required
...
```

In this example we will first have an initial name filter, looking only for workspaces with a name starting with `aws-`. We then add two variable filters to our query. The first filter will require that the workspace has a variable with a key of `mode` and a value containing the string `prod`. The second filter will check the variable with the key of `status`, should it exist, and verify that it does not exactly equal `migrating`. So our resulting dataset would contain only those workspaces starting the with the name `aws-`, containing the string `prod` in the `mode` key, and will not have a `status` of `migrating` should the key exist.

### Tag Filters

Tag filter logic works very similar to variable filter logic and runs before the variable filter. Let's take a look at an example tag filter setup.

```toml
...
[query] # Required but can be left empty

[[query.tags]] # Optional
operator = "NotContains" # Required
name = "team:" # Required
...
```

In this example we will not set a name filter, notice how the `[query]` table is defined but empty. We have one tag filter set which will look at the tags for each workspace in our initial query (since we didn't set any additional filter parameters this would be the first page of workspaces containing up to 20 entries, using the default pagination settings) and will remove any workspace from the results that do not have a tag that contains `team:`.

### Operators

Currently the available "operators" are:

- Equals
  - A variable with the specified key must exist, and must exactly equal the specified value
- NotEquals
  - Should a variable with the specified key exist it must not exactly equal the specified value
- Contains
  - A variable with the specified key must exist, and must contain the specified value
- NotContains
  - Should a variable with the specified key exist it must not contain the specified value
  
## Notes

Rate limiting has been implemented but no retry logic has been added so hitting the limit could result in an error. A maximum rate of 30 requests per second has been set, see [Terraform Docs](https://www.terraform.io/cloud-docs/api-docs#rate-limiting) for information on that limit. Pagination has been implemented but take special care of setting `max_depth` = `0` to pull all pages as this could result in a large number of calls. There are also probably some bugs so use at your own risk!
