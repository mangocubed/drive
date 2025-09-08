use std::collections::HashMap;

use dioxus::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::ValidationErrors;

use crate::presenters::UserPresenter;
use crate::server_fns::ServFnResult;
use crate::utils::run_with_loader;

#[derive(Clone, PartialEq)]
pub struct FormProvider {
    pub callback: Callback<HashMap<String, FormValue>>,
    pub status: ReadOnlySignal<FormStatus>,
}

impl FormProvider {
    pub fn is_pending(&self) -> bool {
        *self.status.read() == FormStatus::Pending
    }
}

#[derive(Clone, Default, Deserialize, PartialEq, Serialize)]
pub enum FormStatus {
    #[default]
    Nothing,
    Pending,
    Success(String, Value),
    Failed(String, ValidationErrors),
}

pub fn use_current_user() -> Resource<Option<UserPresenter>> {
    use_context()
}

pub fn use_form_context() -> FormProvider {
    use_context()
}

pub fn use_form_provider<
    FA: Fn(I) -> R + Copy + 'static,
    I: Clone + DeserializeOwned + 'static,
    R: IntoFuture<Output = ServFnResult<FormStatus>>,
>(
    id: String,
    future: FA,
) -> FormProvider {
    let mut status = use_signal(FormStatus::default);

    let callback = use_callback(move |input: HashMap<String, FormValue>| {
        *status.write() = FormStatus::Pending;
        let id = id.clone();

        spawn(async move {
            let result = run_with_loader(id, move || {
                let input = serde_json::from_value(
                    input
                        .iter()
                        .map(|(name, value)| (name.clone(), value.as_value()))
                        .collect(),
                )
                .expect("Could not get input");

                async move { future(input).await }
            })
            .await;

            match result {
                Ok(response) => {
                    *status.write() = response;
                }
                Err(ServerFnError::ServerError(error)) => {
                    error.run_action();
                }
                _ => (),
            }
        });
    });

    use_context_provider(|| FormProvider {
        callback,
        status: status.into(),
    })
}

pub fn use_resource_with_loader<T, F>(id: String, future: impl FnMut() -> F + Copy + 'static) -> Resource<T>
where
    T: 'static,
    F: Future<Output = T> + 'static,
{
    use_resource(move || run_with_loader(id.clone(), future))
}
