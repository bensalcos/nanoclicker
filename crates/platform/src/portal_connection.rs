//! Register each host D-Bus peer before it makes any portal requests.

pub(crate) async fn connect() -> Result<zbus::Connection, ashpd::Error> {
    let connection = zbus::Connection::session().await?;
    // Registration is once per peer, not once per process. A fresh connection
    // also permits registration with a restarted portal without stale state.
    ashpd::register_host_app_with_connection(
        connection.clone(),
        "io.github.bensalcos.NanoClick".parse()?,
    )
    .await?;
    Ok(connection)
}
