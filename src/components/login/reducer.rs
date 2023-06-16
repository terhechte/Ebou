use std::cell::RefCell;

use crate::environment::{
    model::{Account, AppData, Model, TokenData},
    types::{Instance, User},
    Environment,
};
use navicula::effect::Effect;

pub type ViewStore<'a> = navicula::ViewStore<'a, LoginReducer>;

const FOLLOW_USER_ID: &str = "109325706684051157";

pub struct LoginReducer;


#[derive(Debug, Clone)]
pub enum Selection {
    /// The instance
    Instance(Instance),
    /// The Host url
    Host(String),
}

#[derive(Debug, Clone)]
pub enum LoginAction {
    Load,
    LoadedInstances(Vec<Instance>),
    SelectInstance(Selection),
    ChosenInstance,
    RetrieveUrl(Model, Result<AppData, String>),
    EnteredCode(String),
    ValidatedCode(Result<TokenData, String>),
    RetrievedUser(Result<Account, String>),
    SaveCredentials,
    CloseLogin,

    ActionRegister,
    ActionFollow,
    ActionFollowDone(Result<bool, String>),
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct LoginState {
    pub is_loading: bool,
    pub instances: Vec<Instance>,
    pub selected_instance: Option<Instance>,
    pub selected_instance_url: Option<String>,
    pub app_data: Option<AppData>,
    pub code: Option<String>,
    pub access_token: Option<TokenData>,
    pub model: Option<ModelContainer>,
    pub account: Option<Account>,
    pub error_message: Option<String>,
    pub did_follow: bool,
    pub send_model: RefCell<Option<ModelContainer>>,
    pub done: bool,
    pub close: bool,
}

pub fn reduce<'a>(
    _context: &'a impl navicula::types::MessageContext<LoginAction, LoginAction, ()>,
    action: LoginAction,
    state: &'a mut LoginState,
    environment: &'a Environment,
) -> Effect<'static, LoginAction> {
    log::trace!("{action:?}");
    let instances = environment.instances.clone();
    match action {
        LoginAction::Load => Effect::future(
            async move { instances.search(None).await },
            LoginAction::LoadedInstances,
        ),
        LoginAction::LoadedInstances(instances) => {
            state.instances = instances;
            state.is_loading = false;
            Effect::NONE
        }
        LoginAction::SelectInstance(instance) => {
            match instance {
                Selection::Instance(instance) => state.selected_instance = Some(instance),
                Selection::Host(term) => {
                    state.selected_instance = None;
                    state.selected_instance_url = None;
                    // if the user typed in a URL, try to see if it is a
                    // server
                    if let Ok(success) = url::Url::parse(&term) {
                        state.selected_instance_url = Some(success.as_str().to_string());
                    } else {
                        state.is_loading = true;
                        return Effect::future(
                            async move { instances.search(Some(term)).await },
                            LoginAction::LoadedInstances,
                        );
                    }
                }
            }
            Effect::NONE
        }
        LoginAction::ChosenInstance => {
            let Some(url) = state.selected_instance.as_ref()
            .map(|e| e.url())
            .or(state.selected_instance_url.clone())
            else {
                return Effect::NONE
            };
            let model = Model::new(url, None);
            let cloned = model.clone();
            state.is_loading = true;
            return Effect::future(async move { model.register().await }, move |result| {
                LoginAction::RetrieveUrl(cloned, result)
            });
        }
        LoginAction::RetrieveUrl(model, result) => {
            match result {
                Ok(n) => {
                    state.model = Some(ModelContainer::new(n.id.clone(), model));
                    if let Some(ref url) = n.url {
                        environment.open_url(url)
                    }
                    state.app_data = Some(n);
                }
                Err(e) => state.error_message = Some(format!("Mastodon Error: {e:?}")),
            }
            state.is_loading = false;
            Effect::NONE
        }
        LoginAction::EnteredCode(code) => {
            let Some(data) = state.app_data.as_ref() else {
                return Effect::NONE
            };
            let Some(model) = state.model.as_ref().map(|e| e.cloned()) else {
                return Effect::NONE
            };
            state.is_loading = true;
            let (client_id, client_secret) = (data.client_id.clone(), data.client_secret.clone());
            Effect::future(
                async move { model.authenticate(client_id, client_secret, code).await },
                LoginAction::ValidatedCode,
            )
        }
        LoginAction::ValidatedCode(result) => {
            match result {
                Ok(n) => {
                    state.access_token = Some(n.clone());
                    let Some(model) = state.model.as_ref().map(|e| e.cloned()) else {
                        return Effect::NONE
                    };
                    // create a new model with the new access token
                    let new_model = Model::new(model.url, Some(n.access_token.clone()));
                    state.model = Some(ModelContainer::new(n.access_token.clone() , new_model.clone()));
                    return Effect::future(
                        async move { new_model.login().await },
                        LoginAction::RetrievedUser,
                    );
                }
                Err(e) => state.error_message = Some(format!("{e:?}")),
            }
            state.is_loading = false;
            Effect::NONE
        }
        LoginAction::RetrievedUser(result) => {
            state.is_loading = false;
            match result {
                Ok(account) => {
                    state.account = Some(account);
                    return Effect::action(LoginAction::SaveCredentials);
                }
                Err(error) => state.error_message = Some(format!("{error:?}")),
            }
            Effect::NONE
        }
        LoginAction::SaveCredentials => {
            let Some(token) = state.access_token.clone() else { 
                return Effect::NONE 
            };
            let Some(account) = state.account.clone() else { 
                return Effect::NONE 
            };
            let Some(appdata) = state.app_data.clone() else { 
                return Effect::NONE 
            };
            let Some(ref model) = state.model else {
                return Effect::NONE 
            };

            let user = User::new(model.url(), account, token, appdata);

            if let Err(e) = environment.repository.update_or_insert_user(user) {
                state.error_message = Some(format!("{e:?}"));
            }

            state.done = true;
            state.send_model = RefCell::new(state.model.clone());

            Effect::NONE
        }
        LoginAction::CloseLogin => {
            state.close = true;
            Effect::NONE
        }
        LoginAction::ActionRegister => {
            let Some(ref instance) = state.selected_instance else {
                return Effect::NONE
            };
            let url = format!("https://{}", instance.name);
            environment.open_url(&url);
            Effect::NONE
        }
        LoginAction::ActionFollow => {
            let Some(model) = state.model.as_ref().map(|e| e.cloned()) else {
                return Effect::NONE
            };
            Effect::future(
                async move { model.follow(FOLLOW_USER_ID.to_string()).await },
                LoginAction::ActionFollowDone,
            )
        }
        LoginAction::ActionFollowDone(_) => {
            state.did_follow = true;
            Effect::NONE
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct ModelContainer {
    pub id: String,
    pub model: Model,
}

impl ModelContainer {
    pub fn new(id: String, model: Model) -> Self {
        Self { id, model }
    }

    pub fn cloned(&self) -> Model {
        self.model.clone()
    }

    pub fn url(&self) -> String {
        self.model.url.clone()
    }
}

impl PartialEq for ModelContainer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ModelContainer {}
