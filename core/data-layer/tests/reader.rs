#[cfg(test)]
mod reader {
    use data_layer::reader::yaml_reader::read_yaml_file;

    #[test]
    fn yaml_reader() -> Result<(), Box<dyn std::error::Error>> {
        let result = read_yaml_file("./tests/yaml/example.yaml")?;
        assert_eq!(result.metrics.len(), 1);
        assert_eq!(result.slos.len(), 1);
        println!("{:?}", result);
        Ok(())
    }
}
