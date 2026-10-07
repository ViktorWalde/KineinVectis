//! Encerra o pool `MongoDB` antes de liberar a operacao do destino.
//! Declarar antes de cursores/sessoes: esses recursos devem sair primeiro.

use std::ops::Deref;

use kinein_protocol::DataSourceProfile;
use mongodb::sync::Client;

use super::mongo::{self, MongoFailure};
use super::secret::Secret;

/// Cliente local a uma operacao; Drop aguarda workers e conexoes do driver.
pub(super) struct Connection(Client);

impl Connection {
    /// Cria o cliente sem manter handles de recursos fora deste escopo.
    pub(super) fn connect(
        profile: &DataSourceProfile,
        secret: Option<&Secret>,
    ) -> Result<Self, MongoFailure> {
        Client::with_options(mongo::options_for(profile, secret))
            .map(Self)
            .map_err(|error| mongo::describe(&error, &profile.host))
    }
}

impl Deref for Connection {
    type Target = Client;

    fn deref(&self) -> &Client {
        &self.0
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Clones de Client nao sao handles de recursos; cursores ja' sairam.
        self.0.clone().shutdown().run();
    }
}
