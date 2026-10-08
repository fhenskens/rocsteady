//! Set ROC_ENDPOINT and optionally ROC_OPERATOR / ROC_PASSWORD (numeric legacy password).
use rocsteady::{Client, ClientConfig, Login};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = std::env::var("ROC_ENDPOINT")?;
    let login = match (std::env::var("ROC_OPERATOR"), std::env::var("ROC_PASSWORD")) {
        (Ok(operator), Ok(password)) => Some(Login::new(
            operator.as_bytes().try_into()?,
            password.parse()?,
            None,
        )?),
        (Err(std::env::VarError::NotPresent), Err(std::env::VarError::NotPresent)) => None,
        _ => return Err("set both ROC_OPERATOR and ROC_PASSWORD, or neither".into()),
    };
    let client = Client::tcp(
        endpoint,
        ClientConfig {
            login,
            ..Default::default()
        },
    )?;
    let result = async {
        client.connected().await?;
        println!("configuration: {:?}", client.read_configuration().await?);
        println!("clock: {:?}", client.read_clock().await?);
        Ok::<_, Box<dyn std::error::Error>>(())
    }
    .await;
    client.stop().await;
    result
}
