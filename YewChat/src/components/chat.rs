use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let current_username = user.username.borrow().clone();

        html! {
            <div class="flex w-screen h-screen bg-gray-50">
                // Sidebar with user list
                <div class="flex-none w-72 h-screen bg-white shadow-md flex flex-col">
                    <div class="text-xl p-4 font-bold border-b border-gray-200 bg-blue-600 text-white">
                        <div class="flex items-center gap-2">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                            </svg>
                            {"Online Users"}
                        </div>
                    </div>
                    <div class="overflow-auto flex-grow">
                    {
                        self.users.clone().iter().map(|u| {
                            let is_current_user = u.name == current_username;
                            html!{                                <div class={classes!(
                                    "flex", "items-center", "m-3", "rounded-lg", "p-3", "transition-all", "hover:bg-blue-50", "cursor-pointer",
                                    if is_current_user { vec!["bg-blue-100", "border-l-4", "border-blue-500"] } else { vec!["bg-white"] }
                                )}>
                                    <div class="relative">
                                        <img class="w-12 h-12 rounded-full shadow-sm" src={u.avatar.clone()} alt="avatar"/>
                                        <div class="absolute bottom-0 right-0 w-3 h-3 bg-green-500 rounded-full border-2 border-white"></div>
                                    </div>
                                    <div class="flex-grow ml-3">
                                        <div class="flex text-sm font-medium justify-between">
                                            <div class="flex items-center gap-1">
                                                {u.name.clone()}
                                                {
                                                    if is_current_user {
                                                        html! { <span class="text-xs bg-blue-600 text-white px-2 rounded-full">{"You"}</span> }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                            </div>
                                        </div>
                                        <div class="text-xs text-gray-500 mt-1">
                                            {"Online"}
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                    </div>
                </div>
                
                // Main chat area
                <div class="grow h-screen flex flex-col bg-white shadow-lg">
                    // Chat header
                    <div class="w-full h-16 border-b border-gray-200 bg-white shadow-sm flex items-center px-4">
                        <div class="flex items-center">
                            <div class="text-xl font-semibold flex items-center gap-2">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                                </svg>
                                {"Chat Room"}
                            </div>
                            <div class="ml-3 bg-green-100 text-green-800 text-xs px-2 py-1 rounded-full">
                                {format!("{} users online", self.users.len())}
                            </div>
                        </div>
                    </div>
                    
                    // Messages container with gradient background
                    <div class="w-full flex-grow overflow-auto p-4 bg-gradient-to-b from-blue-50 to-gray-50">
                        {
                            self.messages.iter().map(|m| {
                                let binding = UserProfile { 
                                    name: m.from.clone(), 
                                    avatar: format!("https://avatars.dicebear.com/api/adventurer-neutral/{}.svg", m.from) 
                                };
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap_or(&binding);
                                let is_current_user = m.from == current_username;
                                
                                html!{                                    <div class={classes!(
                                        "flex", "mb-4", "transition-all", "duration-300", "ease-in",
                                        if is_current_user { "justify-end" } else { "justify-start" }
                                    )}>
                                        {
                                            if !is_current_user {
                                                html! {
                                                    <img class="w-10 h-10 rounded-full self-end mr-2 shadow-sm" src={user.avatar.clone()} alt="avatar"/>
                                                }
                                            } else {
                                                html! {}
                                            }
                                        }
                                        <div class={classes!(
                                            "rounded-2xl", "p-4", "max-w-xl", "shadow-sm",                                            if is_current_user {
                                                vec!["bg-blue-600", "text-white", "rounded-br-none"]
                                            } else {
                                                vec!["bg-white", "rounded-bl-none"]
                                            }
                                        )}>
                                            <div class={classes!(
                                                "font-medium", "mb-1",
                                                if is_current_user { vec!["text-blue-100"] } else { vec!["text-gray-800"] }
                                            )}>
                                                {m.from.clone()}
                                            </div>
                                            <div class={classes!(
                                                if is_current_user { vec!["text-white"] } else { vec!["text-gray-700"] }
                                            )}>
                                                {
                                                    if m.message.ends_with(".gif") {
                                                        html!{
                                                            <div class="mt-2 rounded-lg overflow-hidden shadow-sm">
                                                                <img class="w-full" src={m.message.clone()}/>
                                                            </div>
                                                        }
                                                    } else {
                                                        html!{
                                                            <div class="whitespace-pre-wrap break-words">
                                                                {m.message.clone()}
                                                            </div>
                                                        }
                                                    }
                                                }
                                            </div>
                                        </div>
                                        {
                                            if is_current_user {
                                                html! {
                                                    <img class="w-10 h-10 rounded-full self-end ml-2 shadow-sm" src={user.avatar.clone()} alt="avatar"/>
                                                }
                                            } else {
                                                html! {}
                                            }
                                        }
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    
                    // Input area
                    <div class="w-full p-4 border-t border-gray-200 bg-white flex items-center gap-2">
                        <input 
                            ref={self.chat_input.clone()} 
                            type="text" 
                            placeholder="Type your message here..." 
                            class="block w-full py-3 px-4 bg-gray-100 rounded-full outline-none focus:ring-2 focus:ring-blue-500 focus:bg-white transition-all" 
                            name="message" 
                            required=true 
                        />
                        <button 
                            onclick={submit} 
                            class="p-3 bg-blue-600 rounded-full flex justify-center items-center text-white hover:bg-blue-700 transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500"
                        >
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-6 h-6 fill-white">
                                <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}