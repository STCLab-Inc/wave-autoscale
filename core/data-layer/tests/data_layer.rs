/**
 * Now, each test function is responsible for a single assertion,
 * making it easier to identify and fix issues.
 * You can add more test functions in a similar manner to test other aspects of the ParserResult.
 */

#[cfg(test)]
mod data_layer {
    use anyhow::Result;
    use data_layer::data_layer::{DataLayer, DataLayerNewParam};
    #[tokio::test]
    async fn test_data_layer_sqlite() -> Result<()> {
        let data_layer = DataLayer::new(DataLayerNewParam {
            sql_url: "sqlite://./tests/data-layer/test.db".to_string(),
        })
        .await;
        println!("data_layer: {:?}", data_layer);
        Ok(())
    }
}
