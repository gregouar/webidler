use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::storage;

#[derive(Clone, Copy)]
pub struct AuthContext {
    get_jwt: Signal<String>,
    set_jwt: WriteSignal<String>,
}

impl AuthContext {
    pub fn token(&self) -> String {
        self.get_jwt.get()
    }

    pub fn sign_in(&self, token: String) {
        self.set_jwt.set(token);
    }

    pub fn sign_out(&self) {
        let (_, _, del_jwt) = storage::use_local_storage::<String, JsonSerdeCodec>("jwt");
        del_jwt();
    }
}

pub fn provide_auth_context() {
    let (get_jwt, set_jwt, _) = storage::use_local_storage::<String, JsonSerdeCodec>("jwt");
    provide_context(AuthContext { get_jwt, set_jwt });
}
