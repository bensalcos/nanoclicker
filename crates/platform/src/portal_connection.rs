//! Register host D-Bus peers when the frontend supports explicit app identity.

pub(crate) async fn connect() -> Result<zbus::Connection, ashpd::Error> {
    let connection = zbus::Connection::session().await?;
    // Registration is once per peer, not once per process. A fresh connection
    // also permits registration with a restarted portal without stale state.
    let registration = ashpd::register_host_app_with_connection(
        connection.clone(),
        "io.github.bensalcos.NanoClick".parse()?,
    )
    .await;
    match registration {
        Ok(()) => {}
        // Older frontends discover host identity without Registry. Continue to
        // their normal capability/consent checks, but never bypass a refusal
        // from a frontend that implements registration.
        Err(ashpd::Error::PortalNotFound(interface))
            if interface.as_str() == "org.freedesktop.host.portal.Registry" => {}
        Err(ashpd::Error::Zbus(zbus::Error::MethodError(name, ..)))
            if name.as_str() == "org.freedesktop.DBus.Error.UnknownInterface" => {}
        Err(error) => return Err(error),
    }
    Ok(connection)
}
