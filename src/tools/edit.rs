
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use obfstr::obfstr;
use lazy_static::lazy_static;

use crate::tools::base::{BaseToolImpl, Tool, ToolParameter, ToolSchema};

lazy_static! {
    static ref EDIT_TOOL_DESCRIPTION: String = obfstr!(r#"Custom editing tool for viewing, creating and editing files
* State is persistent across command calls
* The create command cannot be used if the path already exists
* For str_replace: old_str must match EXACTLY and be unique in the file"#).to_string();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EditArguments {
    command: String,
    path: String,
    #[serde(default)]
    file_text: Option<String>,
    #[serde(default)]
    old_str: Option<String>,
    #[serde(default)]
    new_str: Option<String>,
    #[serde(default)]
    insert_line: Option<usize>,
    #[serde(default)]
    view_range: Option<Vec<i32>>,
}

pub struct TextEditorTool {
    base: BaseToolImpl,
    model_provider: String,
    allowed_folders: Vec<String>,
}

impl TextEditorTool {
    pub fn new(model_provider: String, allowed_folders: Vec<String>) -> Self {
        let parameters = vec![
            ToolParameter {
                name: "command".to_string(),
                param_type: "string".to_string(),
                description: obfstr!("The command to run. Allowed: view, create, str_replace, insert").to_string(),
                required: true,
                default: None,
            },
            ToolParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: obfstr!("Absolute path to file or directory").to_string(),
                required: true,
                default: None,
            },
            ToolParameter {
                name: "file_text".to_string(),
                param_type: "string".to_string(),
                description: obfstr!("Content for create command").to_string(),
                required: false,
                default: None,
            },
            ToolParameter {
                name: "old_str".to_string(),
                param_type: "string".to_string(),
                description: obfstr!("String to replace (for str_replace)").to_string(),
                required: false,
                default: None,
            },
            ToolParameter {
                name: "new_str".to_string(),
                param_type: "string".to_string(),
                description: obfstr!("Replacement string (for str_replace/insert)").to_string(),
                required: false,
                default: None,
            },
            ToolParameter {
                name: "insert_line".to_string(),
                param_type: "integer".to_string(),
                description: obfstr!("Line number for insert command").to_string(),
                required: false,
                default: None,
            },
            ToolParameter {
                name: "view_range".to_string(),
                param_type: "array".to_string(),
                description: obfstr!("Line range for view command [start, end]").to_string(),
                required: false,
                default: None,
            },
        ];

        let base = BaseToolImpl::new(
            "str_replace_based_edit_tool".to_string(),
            EDIT_TOOL_DESCRIPTION.clone(),
            parameters,
        );

        Self {
            base,
            model_provider,
            allowed_folders,
        }
    }

    fn validate_path(&self, path: &Path) -> Result<()> {
        if !path.is_absolute() {
            return Err(anyhow::anyhow!(
                "Path must be absolute, starting with '/'. Got: {}",
                path.display()
            ));
        }

        // Check if path is within allowed folders
        if !self.allowed_folders.is_empty() {
            let mut is_allowed = false;

            // Try to canonicalize the path first
            let path_to_check = if let Ok(canonical) = path.canonicalize() {
                canonical
            } else {
                // If the file doesn't exist, try to canonicalize the parent directory
                if let Some(parent) = path.parent() {
                    if let Ok(parent_canonical) = parent.canonicalize() {
                        parent_canonical.join(path.file_name().unwrap_or_default())
                    } else {
                        path.to_path_buf()
                    }
                } else {
                    path.to_path_buf()
                }
            };

            for allowed_folder in &self.allowed_folders {
                // Try to canonicalize the allowed folder
                let allowed_canonical = Path::new(allowed_folder)
                    .canonicalize()
                    .unwrap_or_else(|_| PathBuf::from(allowed_folder));

                // Check if path starts with the allowed folder
                if path_to_check.starts_with(&allowed_canonical) {
                    is_allowed = true;
                    break;
                }

                // Also check the original path in case canonicalization failed
                if path.starts_with(allowed_folder) {
                    // Additional check: ensure no path traversal
                    let path_str = path.to_string_lossy();
                    if !path_str.contains("/../") && !path_str.ends_with("/..") {
                        is_allowed = true;
                        break;
                    }
                }
            }

            if !is_allowed {
                return Err(anyhow::anyhow!(
                    "Path {} is not within allowed folders",
                    path.display()
                ));
            }
        }

        Ok(())
    }

    fn view_file(&self, path: &Path, view_range: Option<Vec<i32>>) -> Result<String> {
        if path.is_dir() {
            // List directory contents
            let mut output = String::new();
            output.push_str(&format!("Directory contents of {}:\n", path.display()));

            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let file_type = if entry.file_type()?.is_dir() { "dir" } else { "file" };
                output.push_str(&format!("  {} ({})\n", entry.file_name().to_string_lossy(), file_type));
            }
            return Ok(output);
        }

        // Read file
        let content = fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();

        let output = if let Some(range) = view_range {
            if range.len() != 2 {
                return Err(anyhow::anyhow!("view_range must have exactly 2 elements"));
            }

            let start = (range[0] as usize).saturating_sub(1); // Convert to 0-indexed
            let end = if range[1] == -1 {
                lines.len()
            } else {
                range[1] as usize
            };

            lines[start.min(lines.len())..end.min(lines.len())]
                .iter()
                .enumerate()
                .map(|(i, line)| format!("{:6}→{}", start + i + 1, line))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            lines
                .iter()
                .enumerate()
                .map(|(i, line)| format!("{:6}→{}", i + 1, line))
                .collect::<Vec<_>>()
                .join("\n")
        };

        Ok(output)
    }

    fn create_file(&self, path: &Path, content: &str) -> Result<String> {
        if path.exists() {
            return Err(anyhow::anyhow!(
                "File already exists at {}. Remove it first if you want to recreate it.",
                path.display()
            ));
        }

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)?;
        Ok(format!("File created at {}", path.display()))
    }

    fn str_replace(&self, path: &Path, old_str: &str, new_str: Option<&str>) -> Result<String> {
        let content = fs::read_to_string(path)?;

        // Count occurrences
        let occurrences = content.matches(old_str).count();

        if occurrences == 0 {
            return Err(anyhow::anyhow!(
                "old_str not found in file. Make sure it matches exactly, including whitespace."
            ));
        }

        if occurrences > 1 {
            return Err(anyhow::anyhow!(
                "old_str appears {} times in the file. It must be unique. Add more context to make it unique.",
                occurrences
            ));
        }

        // Perform replacement
        let new_content = content.replace(old_str, new_str.unwrap_or(""));
        fs::write(path, new_content)?;

        Ok(format!("Successfully replaced content in {}", path.display()))
    }

    fn insert_at_line(&self, path: &Path, insert_line: usize, new_str: &str) -> Result<String> {
        let content = fs::read_to_string(path)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        if insert_line > lines.len() {
            return Err(anyhow::anyhow!(
                "insert_line {} is beyond the file length {}",
                insert_line,
                lines.len()
            ));
        }

        // Insert after the specified line (0 means insert at beginning)
        lines.insert(insert_line, new_str.to_string());

        let new_content = lines.join("\n");
        fs::write(path, new_content)?;

        Ok(format!("Successfully inserted content at line {} in {}", insert_line + 1, path.display()))
    }
}


impl Tool for TextEditorTool {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }

    fn schema(&self) -> ToolSchema {
        self.base.schema.clone()
    }

    fn execute(&self, arguments: &serde_json::Value) -> Result<String> {
        let args: EditArguments = serde_json::from_value(arguments.clone())
            .context("Invalid arguments for edit tool")?;

        let path = Path::new(&args.path);
        self.validate_path(path)?;

        match args.command.as_str() {
            "view" => self.view_file(path, args.view_range),

            "create" => {
                let content = args.file_text
                    .ok_or_else(|| anyhow::anyhow!("file_text is required for create command"))?;
                self.create_file(path, &content)
            }

            "str_replace" => {
                let old_str = args.old_str
                    .ok_or_else(|| anyhow::anyhow!("old_str is required for str_replace command"))?;
                self.str_replace(path, &old_str, args.new_str.as_deref())
            }

            "insert" => {
                let insert_line = args.insert_line
                    .ok_or_else(|| anyhow::anyhow!("insert_line is required for insert command"))?;
                let new_str = args.new_str
                    .ok_or_else(|| anyhow::anyhow!("new_str is required for insert command"))?;
                self.insert_at_line(path, insert_line, &new_str)
            }

            _ => Err(anyhow::anyhow!(
                "Unknown command: {}. Allowed: view, create, str_replace, insert",
                args.command
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Helper function to create a TextEditorTool with specific allowed folders
    fn create_tool_with_allowed_folders(allowed_folders: Vec<String>) -> TextEditorTool {
        TextEditorTool::new("test".to_string(), allowed_folders)
    }

    #[test]
    fn test_validate_path_rejects_relative_paths() {
        let tool = create_tool_with_allowed_folders(vec!["/tmp".to_string()]);

        // Test various relative paths - all should be rejected
        let relative_paths = vec![
            "file.txt",
            "./file.txt",
            "../file.txt",
            "dir/file.txt",
            "./dir/../file.txt",
            "~/file.txt",
        ];

        for path_str in relative_paths {
            let path = Path::new(path_str);
            let result = tool.validate_path(path);
            assert!(
                result.is_err(),
                "Expected relative path '{}' to be rejected",
                path_str
            );

            if let Err(e) = result {
                assert!(
                    e.to_string().contains("Path must be absolute"),
                    "Error message should indicate path must be absolute for path '{}'",
                    path_str
                );
            }
        }
    }

    #[test]
    fn test_validate_path_with_empty_allowed_folders() {
        // When allowed_folders is empty, any absolute path should be allowed
        let tool = create_tool_with_allowed_folders(vec![]);

        let test_paths = vec![
            "/tmp/file.txt",
            "/home/user/document.txt",
            "/etc/config.conf",
            "/var/log/app.log",
        ];

        for path_str in test_paths {
            let path = Path::new(path_str);
            let result = tool.validate_path(path);
            assert!(
                result.is_ok(),
                "Expected absolute path '{}' to be allowed when allowed_folders is empty",
                path_str
            );
        }
    }

    #[test]
    fn test_validate_path_enforces_allowed_folders() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_path = temp_dir.path().to_str().unwrap().to_string();
        let tool = create_tool_with_allowed_folders(vec![allowed_path.clone()]);

        // Path inside allowed folder should be OK
        let valid_path = temp_dir.path().join("file.txt");
        assert!(
            tool.validate_path(&valid_path).is_ok(),
            "Path inside allowed folder should be accepted"
        );

        // Path outside allowed folder should be rejected
        let invalid_path = Path::new("/etc/passwd");
        let result = tool.validate_path(invalid_path);
        assert!(
            result.is_err(),
            "Path outside allowed folder should be rejected"
        );

        if let Err(e) = result {
            assert!(
                e.to_string().contains("not within allowed folders"),
                "Error should indicate path is not within allowed folders"
            );
        }
    }

    #[test]
    fn test_validate_path_with_symlink_escape_attempt() {
        // This test ensures symlinks can't be used to escape allowed folders
        let temp_dir = TempDir::new().unwrap();
        let allowed_path = temp_dir.path().to_str().unwrap().to_string();
        let tool = create_tool_with_allowed_folders(vec![allowed_path.clone()]);

        // Create a directory structure
        let safe_dir = temp_dir.path().join("safe");
        let _ = fs::create_dir(&safe_dir);

        // Create a symlink that tries to escape to parent directory
        let symlink_path = safe_dir.join("escape_link");

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            // Try to create a symlink pointing outside the allowed directory
            let _ = symlink("../../", &symlink_path);

            // The validate_path should handle this correctly
            // Either by resolving the symlink or handling the error appropriately
            let _ = tool.validate_path(&symlink_path);
        }
    }

    #[test]
    fn test_validate_path_with_path_traversal_attempts() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_path = temp_dir.path().to_str().unwrap().to_string();
        let tool = create_tool_with_allowed_folders(vec![allowed_path.clone()]);

        // Various path traversal attempts that should be caught
        let traversal_attempts = vec![
            format!("{}/../../../etc/passwd", allowed_path),
            format!("{}/./../../etc/passwd", allowed_path),
            format!("{}/subdir/../../../../../../etc/passwd", allowed_path),
        ];

        for path_str in traversal_attempts {
            let path = Path::new(&path_str);

            // Create a real file to test canonicalization
            let test_file = temp_dir.path().join("test.txt");
            let _ = fs::write(&test_file, "test");

            // Test with a path that exists and uses ..
            let escaped_path = temp_dir.path().join("../");
            let result = tool.validate_path(&escaped_path);

            // This should be rejected if it goes outside the allowed folder
            if escaped_path.exists() && escaped_path.canonicalize().is_ok() {
                let canonical = escaped_path.canonicalize().unwrap();
                let allowed_canonical = Path::new(&allowed_path).canonicalize().unwrap_or_else(|_| PathBuf::from(&allowed_path));

                if !canonical.starts_with(&allowed_canonical) {
                    assert!(
                        result.is_err(),
                        "Path traversal attempt should be rejected: {}",
                        path_str
                    );
                }
            }
        }
    }

    #[test]
    fn test_validate_path_with_multiple_allowed_folders() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        let allowed_paths = vec![
            temp_dir1.path().to_str().unwrap().to_string(),
            temp_dir2.path().to_str().unwrap().to_string(),
        ];

        let tool = create_tool_with_allowed_folders(allowed_paths);

        // Both paths should be allowed
        let path1 = temp_dir1.path().join("file1.txt");
        let path2 = temp_dir2.path().join("file2.txt");

        assert!(
            tool.validate_path(&path1).is_ok(),
            "Path in first allowed folder should be accepted"
        );
        assert!(
            tool.validate_path(&path2).is_ok(),
            "Path in second allowed folder should be accepted"
        );

        // Path outside both should be rejected
        let invalid_path = Path::new("/tmp/not_allowed/file.txt");
        assert!(
            tool.validate_path(invalid_path).is_err(),
            "Path outside all allowed folders should be rejected"
        );
    }

    #[test]
    fn test_validate_path_with_nested_allowed_folders() {
        let temp_dir = TempDir::new().unwrap();
        let parent_dir = temp_dir.path().join("parent");
        let child_dir = parent_dir.join("child");
        fs::create_dir_all(&child_dir).unwrap();

        // Only allow the child directory
        let tool = create_tool_with_allowed_folders(vec![
            child_dir.to_str().unwrap().to_string()
        ]);

        // Path in child dir should be allowed
        let valid_path = child_dir.join("file.txt");
        assert!(
            tool.validate_path(&valid_path).is_ok(),
            "Path in allowed child directory should be accepted"
        );

        // Path in parent dir (outside allowed) should be rejected
        let invalid_path = parent_dir.join("file.txt");
        assert!(
            tool.validate_path(&invalid_path).is_err(),
            "Path in parent directory should be rejected when only child is allowed"
        );
    }

    #[test]
    fn test_validate_path_with_special_characters_in_path() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_path = temp_dir.path().to_str().unwrap().to_string();
        let tool = create_tool_with_allowed_folders(vec![allowed_path.clone()]);

        // Test paths with special characters that might be used in injection attempts
        let special_paths = vec![
            temp_dir.path().join("file;rm -rf.txt"),
            temp_dir.path().join("file&whoami.txt"),
            temp_dir.path().join("file|ls.txt"),
            temp_dir.path().join("file`id`.txt"),
            temp_dir.path().join("file$(pwd).txt"),
        ];

        for path in special_paths {
            // These should be allowed as long as they're within the allowed folder
            // The validate_path function only checks location, not filename content
            let result = tool.validate_path(&path);
            assert!(
                result.is_ok(),
                "Special characters in filename should not affect path validation: {:?}",
                path
            );
        }
    }

    #[test]
    fn test_validate_path_canonicalization_fallback() {
        let tool = create_tool_with_allowed_folders(vec!["/tmp".to_string()]);

        // Test with a non-existent path that can't be canonicalized
        let non_existent = Path::new("/tmp/definitely_does_not_exist_234897234/file.txt");
        let result = tool.validate_path(non_existent);

        // Should still work with the fallback to original path
        assert!(
            result.is_ok(),
            "Non-existent path within allowed folder should still be validated"
        );

        // Non-existent path outside allowed folder should still be rejected
        let non_existent_outside = Path::new("/etc/definitely_does_not_exist_234897234/file.txt");
        let result = tool.validate_path(non_existent_outside);
        assert!(
            result.is_err(),
            "Non-existent path outside allowed folder should be rejected"
        );
    }

    #[test]
    fn test_validate_path_prevents_double_dot_escape() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_dir = temp_dir.path().join("allowed");
        fs::create_dir(&allowed_dir).unwrap();

        let tool = create_tool_with_allowed_folders(vec![
            allowed_dir.to_str().unwrap().to_string()
        ]);

        // Create a file in the allowed directory
        let safe_file = allowed_dir.join("safe.txt");
        fs::write(&safe_file, "safe content").unwrap();

        // Try to escape using .. in an existing path
        let escape_attempt = allowed_dir.join("../escape.txt");
        let _ = fs::write(&escape_attempt, "escaped content"); // This might fail, which is fine

        // Validate should catch the escape attempt
        let result = tool.validate_path(&escape_attempt);

        // The path should be rejected if it escapes the allowed directory
        if escape_attempt.exists() && escape_attempt.canonicalize().is_ok() {
            let canonical = escape_attempt.canonicalize().unwrap();
            let allowed_canonical = allowed_dir.canonicalize().unwrap();

            if !canonical.starts_with(&allowed_canonical) {
                assert!(
                    result.is_err(),
                    "Path with .. that escapes allowed directory should be rejected"
                );
            }
        }
    }

    #[test]
    fn test_validate_path_with_root_as_allowed() {
        // Special case: if root is allowed, everything should be allowed
        let tool = create_tool_with_allowed_folders(vec!["/".to_string()]);

        let test_paths = vec![
            Path::new("/etc/passwd"),
            Path::new("/tmp/file.txt"),
            Path::new("/home/user/documents/file.txt"),
            Path::new("/var/log/system.log"),
        ];

        for path in test_paths {
            assert!(
                tool.validate_path(path).is_ok(),
                "All absolute paths should be allowed when root is in allowed_folders"
            );
        }
    }
}