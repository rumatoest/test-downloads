use derive_more::Display;
use rand::Rng;
use thiserror::Error;
use tokio::time;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Server {0:?}: abruptly disconnected")]
    Disconnected(ServerName),
}

#[derive(Display, Debug)]
#[display(fmt = "Binary[source='{}']", "from.0")]
pub struct Binary {
    #[allow(dead_code)]
    from: ServerName,
}

#[derive(Debug, Clone)]
pub struct ServerName(pub String);

pub async fn download(server_name: ServerName) -> Result<Binary, ServerError> {
    // Want to shift different downlad ticks in time
    let sleep_duration = rand::thread_rng().gen_range(10..=300);
    time::sleep(time::Duration::from_millis(sleep_duration)).await;

    let mut interval = time::interval(time::Duration::from_millis(100));
    for _i in 0..5 {
        interval.tick().await;
        if rand::random::<f32>() < 0.1 {
            return Err(ServerError::Disconnected(server_name.into()));
        }
    }
    Ok(Binary {
        from: server_name.into(),
    })
}
