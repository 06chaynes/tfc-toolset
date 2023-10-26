use crate::{
    error::ToolError,
    settings::{Core, Operators},
    workspace::{Workspace, WorkspaceVariables},
};

pub fn by_variable(
    workspace_variables: &mut Vec<WorkspaceVariables>,
    config: &Core,
) -> Result<(), ToolError> {
    // You have to return a bool from the closure.
    // If you return true, the element is not removed;
    // if you return false, it is removed.
    workspace_variables.retain(|workspace| {
        let mut keep = false;
        if let Some(query) = config.clone().workspaces.query {
            if let Some(variables) = query.variables {
                for variable in variables {
                    match variable.operator {
                        Operators::Equals => {
                            // Should be equal.
                            // so if it's not equal or not present we need to remove.
                            keep = false;
                            for var in workspace.variables.clone() {
                                if var.attributes.key == variable.key {
                                    if let Some(value) = var.attributes.value {
                                        if value == variable.value {
                                            keep = true;
                                        }
                                    }
                                }
                            }
                            if !keep {
                                return keep;
                            }
                        }
                        Operators::NotEquals => {
                            // Should not be equal.
                            // so if it's equal we need to remove.
                            keep = true;
                            for var in workspace.variables.clone() {
                                if var.attributes.key == variable.key {
                                    if let Some(value) = var.attributes.value {
                                        if value == variable.value {
                                            keep = false;
                                        }
                                    }
                                }
                            }
                            if !keep {
                                return keep;
                            }
                        }
                        Operators::Contains => {
                            // Should contain the query value.
                            // so if it doesn't contain the value or isn't present we need to remove.
                            keep = false;
                            for var in workspace.variables.clone() {
                                if var.attributes.key == variable.key {
                                    if let Some(value) = var.attributes.value {
                                        if value
                                            .contains(&variable.value.clone())
                                        {
                                            keep = true;
                                        }
                                    }
                                }
                            }
                            if !keep {
                                return keep;
                            }
                        }
                        Operators::NotContains => {
                            // Should be a regex hit.
                            // so if it's not a hit or not present we need to remove.
                            keep = true;
                            for var in workspace.variables.clone() {
                                if var.attributes.key == variable.key {
                                    if let Some(value) = var.attributes.value {
                                        if value
                                            .contains(&variable.value.clone())
                                        {
                                            keep = false;
                                        }
                                    }
                                }
                            }
                            if !keep {
                                return keep;
                            }
                        }
                    }
                }
            }
        }
        keep
    });
    Ok(())
}

pub fn by_tag(
    workspaces: &mut Vec<Workspace>,
    config: &Core,
) -> Result<(), ToolError> {
    // You have to return a bool from the closure.
    // If you return true, the element is not removed;
    // if you return false, it is removed.
    workspaces.retain(|workspace| {
        let mut keep = false;
        if let Some(query) = config.clone().workspaces.query {
            if let Some(tags) = query.tags {
                for q_tag in tags {
                    match q_tag.operator {
                        // Equals and NotEquals is now handled in the url params so could be changed
                        Operators::Equals => {
                            // Should be equal.
                            // so if it's not equal or not present we need to remove.
                            keep = false;
                            if let Some(tag_names) =
                                workspace.attributes.tag_names.clone()
                            {
                                for tag in tag_names {
                                    if tag == q_tag.name {
                                        keep = true;
                                    }
                                }
                            }
                            if !keep {
                                return keep;
                            }
                        }
                        Operators::NotEquals => {
                            // Should not be equal.
                            // so if it's equal we need to remove.
                            keep = true;
                            if let Some(tag_names) =
                                workspace.attributes.tag_names.clone()
                            {
                                for tag in tag_names {
                                    if tag == q_tag.name {
                                        keep = false;
                                    }
                                }
                            }
                            if !keep {
                                return keep;
                            }
                        }
                        Operators::Contains => {
                            // Should contain the query value.
                            // so if it doesn't contain the value or isn't present we need to remove.
                            keep = false;
                            while let Some(tag) =
                                workspace.attributes.tag_names.clone()
                            {
                                if tag.contains(&q_tag.name) {
                                    keep = true;
                                }
                            }
                            if !keep {
                                return keep;
                            }
                        }
                        Operators::NotContains => {
                            // Should be a regex hit.
                            // so if it's not a hit or not present we need to remove.
                            keep = true;
                            while let Some(tag) =
                                workspace.attributes.tag_names.clone()
                            {
                                if tag.contains(&q_tag.name) {
                                    keep = false;
                                }
                            }
                            if !keep {
                                return keep;
                            }
                        }
                    }
                }
            }
        }
        keep
    });
    Ok(())
}
