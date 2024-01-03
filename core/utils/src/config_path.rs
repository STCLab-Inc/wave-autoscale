use std::{env, path::Path};

/**
 * Find the file in specific directories
 *
 * At first, it tries to find the file in the current directory.
 * But if it fails, it tries to find the file in "config" directory under the current directory.
 * If it fails again, it returns Err.
 */
pub fn find_file_in_wa(file_name: &str) -> Result<String, String> {
    let Ok(current_dir) = env::current_dir() else {
        return Err("Error getting current directory".to_string());
    };
    let Some(current_dir) = current_dir.to_str() else {
        return Err("Error getting current directory".to_string());
    };

    let search_directories = vec!["", "/config", "/tests", "/tests/config"]; // Add more directories here

    for dir in search_directories {
        let file_path = format!("{}{}/{}", current_dir, dir, file_name);
        if Path::new(&file_path).exists() {
            return Ok(file_path);
        }
    }

    Err(format!("File not found: {}", file_name))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_file_in_wa_existing_file() {
        let file_name = "wave-config.yaml";
        let result = find_file_in_wa(file_name);
        assert!(result.is_ok());
        let file_path = result.unwrap();
        assert_eq!(
            file_path,
            format!(
                "{}/tests/config/{}",
                env::current_dir().unwrap().to_str().unwrap(),
                file_name
            )
        );
    }

    #[test]
    fn test_find_file_in_wa_non_existing_file() {
        let file_name = "non_existing.txt";
        let result = find_file_in_wa(file_name);
        assert!(result.is_err());
        let error_message = result.unwrap_err();
        assert_eq!(error_message, format!("File not found: {}", file_name));
    }
}
