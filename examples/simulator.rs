use rocsteady::{
    Client, ClientConfig, DataType, Parameter, Tlp, Value,
    simulator::{Model, Simulator},
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = Model::default();
    let tlp = Tlp::new(103, 16, 3);
    model.parameters.insert(tlp, Value::Float(12.5));
    let server = Simulator::bind("127.0.0.1:0".parse()?, model).await?;
    let client = Client::tcp(server.endpoint().to_string(), ClientConfig::default())?;
    let readings = async {
        client.connected().await?;
        let configuration = client.read_configuration().await?;
        println!(
            "device type: {}, mode: {:?}, configuration revision: {}",
            configuration.device_type(),
            configuration.mode(),
            configuration.revision()
        );
        println!("clock: {:?}", client.read_clock().await?);
        println!(
            "parameters: {:?}",
            client
                .read_parameters(&[Parameter::new(tlp, DataType::Float)])
                .await?
        );
        Ok::<_, Box<dyn std::error::Error>>(())
    }
    .await;
    client.stop().await;
    server.stop().await?;
    readings
}
