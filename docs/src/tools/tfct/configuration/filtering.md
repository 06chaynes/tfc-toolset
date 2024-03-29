# Filtering

## Query Filters

Filters can be combined as needed. The current run order for filter logic is:

1. Name
2. Wildcard Name
3. Tags
4. Variables

### Variable Filters

First let's take a look at an example variable filter setup.

```toml
[query]
name = "aws-"
wildcard_name = "*-prod"

[[query.variables]]
key = "mode"
operator = "Contains"
value = "prod"

[[query.variables]]
key = "status"
operator = "NotEqual"
value = "migrating"
```

In this example we will first have an initial name filter, fuzzy searching for workspaces with a name matching `aws-`.
We also have a wildcard name filter, looking for workspaces with a name ending with `-prod`.
We then add two variable filters to our query.
The first filter will require that the workspace has a variable with a key of `mode` and a value containing the string `prod`.
The second filter will check the variable with the key of `status`, should it exist, and verify that it does not exactly equal `migrating`.
So our resulting dataset would contain only those workspaces starting the with the name `aws-`,
containing the string `prod` in the `mode` key, and will not have a `status` of `migrating` should the key exist.

### Tag Filters

Tag filter logic works very similar to variable filter logic and runs before the variable filter. Let's take a look at an example tag filter setup.

```toml
[query]

[[query.tags]]
operator = "NotContains"
name = "team:"
```

In this example we will not set a name filter, notice how the `[query]` table is defined but empty.
We have one tag filter set which will look at the tags for each workspace in our initial query
(since we didn't set any additional filter parameters this would be the first page of workspaces containing up to 20 entries,
using the default pagination settings) and will remove any workspace from the results that do not have a tag that contains `team:`.

### Operators

Currently the available "operators" are:

- Equals
  - A variable with the specified key must exist, and must exactly equal the specified value
  - A tag must exist with a name that exactly equals the specified value
- NotEquals
  - Should a variable with the specified key exist it must not exactly equal the specified value
  - A tag must not exist with a name that exactly equals the specified value
- Contains
  - A variable with the specified key must exist, and must contain the specified value
  - A tag must exist with a name that contains the specified value
- NotContains
  - Should a variable with the specified key exist it must not contain the specified value
  - A tag must not exist with a name that contains the specified value
