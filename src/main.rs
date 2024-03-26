use std::env;
use std::fs;
use std::path::Path;
use tempfile::NamedTempFile;
use std::io::{self, Write};

fn main() {
    // Get the first argument as the directory path
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: clipfil <path>");
        std::process::exit(1);
    }
    let path = &args[1];

    // Traverse the directory and get all non-binary file contents
    let mut file_contents = Vec::new();
    if let Err(e) = visit_dirs(Path::new(path), &mut file_contents) {
        eprintln!("Error when reading directory: {}", e);
        std::process::exit(1);
    }

    // Concatenate all file contents into a single string
    let result = file_contents.join("\n\n");

    // Print the result to verify the contents before writing to the temporary file
    println!("Result to be written to the temporary file:\n{}", result);

    // Write the result to a temporary file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file");
    writeln!(temp_file, "{}", result).expect("Failed to write to temporary file");
    let temp_path = temp_file.into_temp_path();
    println!("The file paths and contents have been written to a temporary file: {}", temp_path.display());

    // Keep the temporary file until the user presses Enter
    println!("Press Enter to finish...");
    let _ = io::stdin().read_line(&mut String::new());

    // Persist the temporary file
    match temp_path.persist("/tmp/clipfil_output.txt") {
        Ok(_) => {
            println!("The temporary file has been persisted at: /tmp/clipfil_output.txt");
        },
        Err(e) => {
            eprintln!("Failed to persist temporary file: {}", e);
        },
    }
}

// Function to traverse directories recursively and read non-binary file contents
fn visit_dirs(dir: &Path, file_contents: &mut Vec<String>) -> Result<(), std::io::Error> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            println!("Visiting path: {}", path.display()); // Added print statement
            if path.is_dir() {
                visit_dirs(&path, file_contents)?;
            } else {
                // Attempt to read the file as text and add to file_contents if successful
                match fs::read_to_string(&path) {
                    Ok(contents) if !contents.is_empty() => {
                        println!("Reading file: {}", path.display()); // Added print statement
                        println!("File contents: {}", contents); // Added print statement
                        file_contents.push(format!("File: {}\n\n{}", path.display(), contents));
                    },
                    Ok(_) => {
                        println!("Skipping empty or binary file: {}", path.display()); // Added print statement
                    },
                    Err(e) => {
                        println!("Skipping binary file: {} (error: {})", path.display(), e); // Added print statement
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    fn setup_test_directory(dir_name: &str, files: Vec<(&str, &str)>) -> std::io::Result<PathBuf> {
        let dir = Path::new(dir_name);
        fs::create_dir_all(dir)?;
        for (file_name, content) in files {
            let parts: Vec<&str> = file_name.split('/').collect();
            let file_path = parts[..parts.len() - 1].iter().fold(dir.to_path_buf(), |acc, &part| acc.join(part));
            fs::create_dir_all(&file_path)?;
            let file_path = file_path.join(parts[parts.len() - 1]);
            if file_name.ends_with(".bin") {
                let binary_content = [0_u8, 159, 146, 150]; // Non-UTF-8 bytes
                let mut file = File::create(file_path)?;
                file.write_all(&binary_content)?;
            } else {
                let mut file = File::create(file_path)?;
                writeln!(file, "{}", content)?;
            }
        }
        Ok(dir.to_path_buf())
    }

    #[test]
    fn test_visit_dirs_with_non_binary_files() {
        let test_dir = setup_test_directory("test_dir", vec![("test.txt", "Test text content")]).unwrap();
        let mut file_contents = Vec::new();
        visit_dirs(&test_dir, &mut file_contents).unwrap();
        assert!(!file_contents.is_empty(), "File contents should not be empty");
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_visit_dirs_with_binary_file() {
        let test_dir = setup_test_directory("test_dir_with_binary", vec![("test.bin", "binary")]).unwrap();
        let mut file_contents = Vec::new();
        visit_dirs(&test_dir, &mut file_contents).unwrap();
        assert!(file_contents.is_empty(), "File contents should be empty when visiting a directory with only binary files");
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_visit_dirs_with_mixed_content() {
        let test_dir = setup_test_directory("test_dir_mixed", vec![("test.txt", "Test text content"), ("test.bin", "binary")]).unwrap();
        let mut file_contents = Vec::new();
        visit_dirs(&test_dir, &mut file_contents).unwrap();
        assert_eq!(file_contents.len(), 1, "File contents should contain only one entry for the non-binary file");
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_visit_dirs_with_empty_directory() {
        let test_dir = setup_test_directory("test_dir_empty", vec![]).unwrap();
        let mut file_contents = Vec::new();
        visit_dirs(&test_dir, &mut file_contents).unwrap();
        assert!(file_contents.is_empty(), "File contents should be empty when visiting an empty directory");
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_visit_dirs_with_nested_directories() {
        let test_dir = setup_test_directory("test_dir_nested", vec![("dir1/test.txt", "Test text content"), ("dir2/test.bin", "binary")]).unwrap();
        let mut file_contents = Vec::new();
        visit_dirs(&test_dir, &mut file_contents).unwrap();
        assert_eq!(file_contents.len(), 1, "File contents should contain only one entry for the non-binary file in nested directory");
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_visit_dirs_with_large_number_of_files() {
        let mut file_names = Vec::new();
        let mut files = Vec::new();
        for i in 0..1000 {
            file_names.push(format!("file{}.txt", i));
        }
        for file_name in file_names.iter() {
            files.push((file_name.as_str(), "Some content"));
        }
        let test_dir = setup_test_directory("test_dir_large", files).unwrap();
        let mut file_contents = Vec::new();
        visit_dirs(&test_dir, &mut file_contents).unwrap();
        assert_eq!(file_contents.len(), 1000, "File contents should contain entries for all non-binary files");
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_visit_dirs_with_deep_nesting() {
        let test_dir = setup_test_directory("test_dir_deep", vec![("dir1/dir2/dir3/test.txt", "Nested content")]).unwrap();
        let mut file_contents = Vec::new();
        visit_dirs(&test_dir, &mut file_contents).unwrap();
        assert_eq!(file_contents.len(), 1, "File contents should contain one entry for the deeply nested non-binary file");
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_visit_dirs_with_varied_encodings() {
        let test_dir = setup_test_directory("test_dir_encodings", vec![("utf8.txt", "UTF-8 content"), ("latin1.txt", "Latin1 content")]).unwrap();
        let mut file_contents = Vec::new();
        visit_dirs(&test_dir, &mut file_contents).unwrap();
        assert_eq!(file_contents.len(), 2, "File contents should contain entries for files with different encodings");
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_visit_dirs_with_permission_error() {
        use std::os::unix::fs::PermissionsExt;

        let test_dir = setup_test_directory("test_dir_no_permission", vec![("no_read.txt", "No read permission")]).unwrap();
        let perms = fs::metadata(&test_dir).unwrap().permissions();
        let mut perms_clone = perms.clone(); // Clone perms before modifying
        perms_clone.set_mode(0o000); // Remove all permissions
        fs::set_permissions(&test_dir, perms_clone).unwrap(); // Use cloned perms to change permissions

        let mut file_contents = Vec::new();
        let result = visit_dirs(&test_dir, &mut file_contents);
        assert!(result.is_err(), "Should error when visiting a directory with no read permission");

        // Reset permissions to remove the directory
        let mut perms_reset = perms; // Use original perms to reset
        perms_reset.set_mode(0o755);
        fs::set_permissions(&test_dir, perms_reset).unwrap();
    }

    #[test]
    fn test_visit_dirs_with_file_no_read_permission() {
        let test_dir = setup_test_directory("test_dir_file_no_permission", vec![("no_read.txt", "No read permission")]).unwrap();
        let file_path = test_dir.join("no_read.txt");
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o000); // Remove all permissions
        fs::set_permissions(&file_path, perms).unwrap();

        let mut file_contents = Vec::new();
        let result = visit_dirs(&test_dir, &mut file_contents);
        assert!(result.is_err(), "Should error when visiting a file with no read permission");

        // Reset permissions to remove the file
        perms.set_mode(0o644);
        fs::set_permissions(&file_path, perms).unwrap();
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_visit_dirs_with_special_characters() {
        let test_dir = setup_test_directory("test_dir_special_chars", vec![("test_@!#&().txt", "Special characters in file name")]).unwrap();
        let mut file_contents = Vec::new();
        visit_dirs(&test_dir, &mut file_contents).unwrap();
        assert_eq!(file_contents.len(), 1, "File contents should contain one entry for the file with special characters");
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_visit_dirs_with_empty_file() {
        let test_dir = setup_test_directory("test_dir_empty_file", vec![("empty.txt", "")]).unwrap();
        let mut file_contents = Vec::new();
        visit_dirs(&test_dir, &mut file_contents).unwrap();
        assert!(file_contents.is_empty(), "File contents should be empty when visiting a directory with an empty file");
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_visit_dirs_with_mixed_binary_non_binary_files() {
        let test_dir = setup_test_directory("test_dir_mixed_files", vec![("test.txt", "Test text content"), ("test.bin", "binary content")]).unwrap();
        let mut file_contents = Vec::new();
        visit_dirs(&test_dir, &mut file_contents).unwrap();
        assert_eq!(file_contents.len(), 1, "File contents should contain one entry for the non-binary file and skip the binary file");
        fs::remove_dir_all(test_dir).unwrap();
    }
}
