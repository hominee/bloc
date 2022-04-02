use crate::web::app::*;
use crate::web::event_bus::{EventBus, Request};
use crate::{constant::*, secp256k1::*, transaction::*};
use std::rc::Rc;
use web_sys::{HtmlElement, HtmlInputElement, HtmlTextAreaElement};
use yew::{html::*, prelude::*};
use yew_agent::{Dispatched, Dispatcher};

#[derive(Properties, PartialEq, Clone)]
pub struct HomePageProps {
    pub users: Rc<Vec<UserMeta>>,
}

#[derive(Clone)]
pub struct Refs {
    sign_passed: bool,
    node_ref_user: NodeRef,
    node_ref_from: NodeRef,
    node_ref_to: NodeRef,
    node_ref_amount: NodeRef,
    node_ref_tips: NodeRef,
    node_ref_secret: NodeRef,
    node_ref_signature: NodeRef,
    node_ref_help_user: NodeRef,
    node_ref_help_from: NodeRef,
    node_ref_help_to: NodeRef,
    node_ref_help_amount: NodeRef,
    node_ref_help_tips: NodeRef,
    node_ref_help_secret: NodeRef,
    node_ref_label_signature: NodeRef,
    node_ref_no_user: NodeRef,
}

pub struct HomePage {
    event_bus: Dispatcher<EventBus>,
    refs: Refs,
    transaction: Option<(Transaction, SecKey)>,
}

pub enum Msg {
    UserCreate(String),
    UserDataCheck,
    InvalidOrNullUserName,
    InvalidOrNullFromPubkey,
    InvalidOrNullToPubkey,
    InvalidOrNullAmount,
    InvalidOrNullTips,
    InvalidOrNullSecret,
    TransferDataCheck,
    TransferSign,
    Transfer(Transaction),
    TransferClean,
    TransferConfirm,
    TransferCleanContent,
}

impl Component for HomePage {
    type Message = Msg;
    type Properties = HomePageProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            transaction: None,
            event_bus: EventBus::dispatcher(),
            refs: Refs {
                sign_passed: false,
                node_ref_user: NodeRef::default(),
                node_ref_from: NodeRef::default(),
                node_ref_to: NodeRef::default(),
                node_ref_amount: NodeRef::default(),
                node_ref_tips: NodeRef::default(),
                node_ref_secret: NodeRef::default(),
                node_ref_signature: NodeRef::default(),
                node_ref_help_user: NodeRef::default(),
                node_ref_help_from: NodeRef::default(),
                node_ref_help_to: NodeRef::default(),
                node_ref_help_amount: NodeRef::default(),
                node_ref_help_tips: NodeRef::default(),
                node_ref_help_secret: NodeRef::default(),
                node_ref_label_signature: NodeRef::default(),
                node_ref_no_user: NodeRef::default(),
            },
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UserDataCheck => {
                if let Some(input_user_name) = self.refs.node_ref_user.cast::<HtmlInputElement>() {
                    let user_name = input_user_name.value();
                    if user_name.is_empty() {
                        log::trace!("none user name");
                        ctx.link().send_message(Msg::InvalidOrNullUserName);
                        return false;
                    }
                    log::info!("got user name: {}", user_name);
                    ctx.link().send_message(Msg::UserCreate(user_name));
                    return false;
                } else {
                    log::trace!("none user name");
                    ctx.link().send_message(Msg::InvalidOrNullUserName);
                    return false;
                }
            }
            Msg::InvalidOrNullUserName => {
                log::trace!("msg InvalidOrNullUserName received");
                if let Some(ele) = self.refs.node_ref_help_user.cast::<HtmlElement>() {
                    ele.set_class_name("help is-danger")
                }
            }
            Msg::UserCreate(user_name) => {
                log::trace!("msg UserCreate received");
                self.event_bus.send(Request::UserCreate(user_name));
                return false;
            }
            Msg::TransferDataCheck => {
                log::trace!("msg TransferDataCheck received");
                if let Some(input_transfer_from) =
                    self.refs.node_ref_from.cast::<HtmlInputElement>()
                {
                    let pubkey_from = input_transfer_from.value().trim().to_owned();
                    if pubkey_from.is_empty() {
                        ctx.link().send_message(Msg::InvalidOrNullFromPubkey);
                        return true;
                    }

                    // check and get desitination's pubkey
                    if let Some(input_transfer_to) =
                        self.refs.node_ref_to.cast::<HtmlInputElement>()
                    {
                        let pubkey_to = input_transfer_to.value().trim().to_owned();
                        if pubkey_to.is_empty() {
                            ctx.link().send_message(Msg::InvalidOrNullToPubkey);
                            return true;
                        }

                        // check and get ampunt
                        if let Some(input_transfer_amount) =
                            self.refs.node_ref_amount.cast::<HtmlInputElement>()
                        {
                            let amount = input_transfer_amount.value_as_number();
                            if amount == 0.0 || amount.is_nan() {
                                ctx.link().send_message(Msg::InvalidOrNullAmount);
                                return true;
                            }

                            // check and get tips
                            if let Some(input_transfer_tips) =
                                self.refs.node_ref_tips.cast::<HtmlInputElement>()
                            {
                                let tips = input_transfer_tips.value_as_number();
                                if tips.is_nan() {
                                    ctx.link().send_message(Msg::InvalidOrNullTips);
                                    return true;
                                }
                                let from = PubKey::from_hex(&pubkey_from);
                                if let Ok(from) = from {
                                    if let Ok(to) = PubKey::from_hex(&pubkey_to) {
                                        if let Some(input_transfer_seckey) =
                                            self.refs.node_ref_secret.cast::<HtmlInputElement>()
                                        {
                                            let hex_sec_key =
                                                input_transfer_seckey.value().trim().to_owned();
                                            let sec_key = SecKey::from_hex(&hex_sec_key);
                                            if sec_key.is_err() {
                                                ctx.link().send_message(Msg::InvalidOrNullSecret);
                                                return true;
                                            }
                                            let sec_key = sec_key.unwrap();
                                            log::info!(
                                                "from: {:?}, to: {:?}, amount: {:?}, tips: {:?}",
                                                from.to_hex(),
                                                to.to_hex(),
                                                amount,
                                                tips
                                            );
                                            let transfer =
                                                Transaction::new(from, to, amount, Some(tips));
                                            self.transaction = Some((transfer, sec_key));
                                            ctx.link().send_message(Msg::TransferSign);

                                            return true;
                                        } else {
                                            ctx.link().send_message(Msg::InvalidOrNullSecret);
                                            return true;
                                        }
                                    } else {
                                        ctx.link().send_message(Msg::InvalidOrNullToPubkey);
                                        return true;
                                    }
                                } else {
                                    ctx.link().send_message(Msg::InvalidOrNullFromPubkey);
                                    return true;
                                }
                            } else {
                                ctx.link().send_message(Msg::InvalidOrNullTips);
                                return true;
                            }
                        } else {
                            ctx.link().send_message(Msg::InvalidOrNullAmount);
                            return true;
                        }
                    } else {
                        ctx.link().send_message(Msg::InvalidOrNullToPubkey);
                        return true;
                    }
                } else {
                    log::trace!("none from");
                    ctx.link().send_message(Msg::InvalidOrNullFromPubkey);
                    return true;
                }
            }
            Msg::InvalidOrNullAmount => {
                log::trace!("msg InvalidOrNullAmount received");
                if let Some(ele) = self.refs.node_ref_help_amount.cast::<HtmlElement>() {
                    ele.set_class_name("help is-danger")
                }
            }
            Msg::InvalidOrNullToPubkey => {
                log::trace!("msg InvalidOrNullToPubkey received");
                if let Some(ele) = self.refs.node_ref_help_to.cast::<HtmlElement>() {
                    ele.set_class_name("help is-danger")
                }
            }
            Msg::InvalidOrNullFromPubkey => {
                log::trace!("msg InvalidOrNullFromPubkey received");
                if let Some(ele) = self.refs.node_ref_help_from.cast::<HtmlElement>() {
                    ele.set_class_name("help is-danger")
                }
            }
            Msg::InvalidOrNullTips => {
                log::trace!("msg InvalidOrNullTips received");
                if let Some(ele) = self.refs.node_ref_help_tips.cast::<HtmlElement>() {
                    ele.set_class_name("help is-danger")
                }
            }
            Msg::TransferSign => {
                if self.transaction.is_some() {
                    if self.refs.sign_passed {
                        log::debug!("already has a signature");
                        return false;
                    }
                    // sign the Transaction
                    let (mut trans, sec_key) = self.transaction.take().unwrap();
                    let key_pair = KeyPair::from(&sec_key);
                    trans.sign(&key_pair);
                    self.transaction = Some((trans, sec_key));
                    self.refs.sign_passed = true;
                    self.show_signature_controler();
                    log::trace!("msg Transfer Signed");
                    return true;
                }
                ctx.link().send_message(Msg::TransferDataCheck);
                return true;
            }
            Msg::TransferConfirm => {
                assert!(self.transaction.is_some(), "transaction must not be null");
                let trans = self.transaction.take().unwrap().0;
                ctx.link().send_message(Msg::Transfer(trans));
            }
            Msg::Transfer(trans) => {
                log::trace!("msg Transfer received");
                self.event_bus.send(Request::Transfer(trans));
                self.refs.sign_passed = false;
                self.transaction = None;
                ctx.link().send_message(Msg::TransferCleanContent);
                self.show_signature_controler();
                return true;
            }
            Msg::TransferClean => {
                self.transaction = None;
                self.refs.sign_passed = false;
                self.show_signature_controler();
                return true;
            }
            Msg::TransferCleanContent => {
                self.refs.clear_input_content();
                return true;
            }
            _ => {}
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link().clone();
        let link0 = ctx.link().clone();
        let link1 = ctx.link().clone();
        let this = self.refs.clone();
        let this_link = ctx.link().clone();
        let this0 = self.refs.clone();
        let this1 = self.refs.clone();
        let this1_link = ctx.link().clone();
        let this2 = self.refs.clone();
        let this2_link = ctx.link().clone();
        let this3 = self.refs.clone();
        let this3_link = ctx.link().clone();
        let this4 = self.refs.clone();
        let this4_link = ctx.link().clone();
        let this5 = self.refs.clone();
        let sign_passed = self.refs.sign_passed;

        html! {
            <>
                <aside class="aside-buttons buttons aside-border">
                    <div class="space-between" style="height:25px;">
                        <span class="title is-6">{ "Initials" }</span>
                        <span class="balance">{ "10000.0" }</span>
                    </div>
                    <div class="space-between" style="height:25px;">
                        <span class="title is-6">{ "Reward" }</span>
                        <span class="balance">{ format!( "{:.1}", *REWARD ) }</span>
                    </div>
                    <hr class="doted mt-3 mb-3" style="margin:0;" />

                    //<form action="">
                        <div class="field">
                            <label for="user-name" class="title is-5">{ "Sign Up" }</label>
                            <div class="control">
                                <input class="input" type="text" onfocus={move |_| this0.clear_help_message("user" )} ref={self.refs.node_ref_user.clone()} id="user-name" name="user-name" value="" placeholder="your name here" />
                                <p class="help no-display" ref={self.refs.node_ref_help_user.clone()} > {"invalid or deplicated user name"} </p>
                            </div>
                            <div class="control mt-1">
                                <button type="submit" onclick={ move |_| link.clone().send_message(Msg::UserDataCheck) } class="button is-info is-small">{ "Create" }</button>
                            </div>
                        </div>
                    //</form>
                    <hr class="doted mt-3 mb-3" style="margin:0;" />

                    <p class="title is-5">{ "Transfer" }</p>
                    //<form >
                        <div class="field">
                            <label for="from-user">{ "From" }</label>
                            <div class="control">
                                <input class="input"  type="text" oninput={move |_| this_link.send_message(Msg::TransferClean) } onfocus={move |_| this.clear_help_message("transfer")} ref={self.refs.node_ref_from.clone()} id="public-key-from" name="public-key-from" placeholder="public key(hex)" />
                                <p class="help no-display" ref={self.refs.node_ref_help_from.clone()} > {"66 long hex string"} </p>
                            </div>
                            <label for="to-user">{ "To" }</label>
                            <div class="control">
                                <input class="input"  type="text" oninput={move |_| this1_link.send_message(Msg::TransferClean) } onfocus={move |_| this1.clear_help_message("transfer") } ref={self.refs.node_ref_to.clone()} id="public-key-to" name="public-key-to" placeholder="public key(hex)" />
                                <p class="help no-display" ref={self.refs.node_ref_help_to.clone()} > {"66 long hex string"} </p>
                            </div>
                            <label for="amount-user">{ "Amount" }</label>
                            <div class="control">
                                <input class="input"  type="number" oninput={move |_| this2_link.send_message(Msg::TransferClean) } onfocus={move |_| this2.clear_help_message("transfer") } ref={self.refs.node_ref_amount.clone()} id="amount" name="amount" min="0" step="any" />
                                <p class="help no-display" ref={self.refs.node_ref_help_amount.clone()} > {"invalid amount"} </p>
                            </div>
                            <label for="tips-user">{ "Tips" }</label>
                            <div class="control">
                                <input class="input"  type="number" oninput={move |_| this3_link.send_message(Msg::TransferClean) } onfocus={move |_| this3.clear_help_message("transfer" ) } ref={self.refs.node_ref_tips.clone()} id="tips" name="tips" min="0" step="any" />
                                <p class="help no-display" ref={self.refs.node_ref_help_tips.clone()} > {"invalid tips"} </p>
                            </div>
                            <label for="from-user">{ "SecretKey" }</label>
                            <div class="control">
                                <input class="input"  type="text" oninput={move |_| this4_link.send_message(Msg::TransferClean) } onfocus={move |_| this4.clear_help_message("transfer" )} ref={self.refs.node_ref_secret.clone()} id="secret-key" name="secret-key" placeholder="secret key(hex)" />
                                <p class="help no-display" ref={self.refs.node_ref_help_secret.clone()} > {"64 long hex string"} </p>
                            </div>
                            <label for="signature" class="no-display" ref={self.refs.node_ref_label_signature.clone() } >{ "Signature" }</label>
                            <div class="control">
                                <textarea  ref={self.refs.node_ref_signature.clone()} style="border-radius: 5px;" class="textarea is-small no-display" placeholder="Signature Here"> </textarea>
                            </div>

                            <div class="control mt-1 space-between">
                                <button onclick={ move |_| {
                                    let msg = if sign_passed {
                                        Msg::TransferClean
                                    } else {
                                        Msg::TransferSign
                                    };
                                    link1.send_message(msg);
                                }} class="button is-info is-small" >{
                                    if !sign_passed { html!{ "Sign" } } else {
                                        html!{ "Cancel" }
                                    }
                                }</button>
                                <button  type="submit" onclick={ move |_| link0.send_message(Msg::TransferConfirm)} class="button is-info is-small" disabled={ !self.refs.sign_passed}>{ "Confirm" }</button>
                            </div>
                        </div>
                    //</form>
                </aside>

            <div class="main-content">
                <div class="user-cards">
                    {
                        if ctx.props().users.len() <= 1 {
                            html!{
                                <div class="notification text-center is-light"  ref={this5.node_ref_no_user}>
                                    <button class="delete" onclick={self.refs.rm_notification()} ></button>
                                    { "No User Created So Far" }
                                </div>
                            }
                        } else {
                                 self.view_users(ctx)
                        }
                    }
                </div>

                <footer class="footer mt-3" >
                    <div class="content has-text-centered">
                        <p> {"A very "}
                        <strong>{ "bare-bone "}</strong>
                        { "Block Chain demo that purely implemented by"}
                        <a href="https://rust-lang.org">{ " Rust" }</a>
                        {" and "}
                        <a href="https://yew.rs">{ "yew " }</a>
                        { "and " }
                            <a href="https://github.com/homelyguy">{ "Bruce Yuan" }</a>
                        </p>
                    </div>
                </footer>

        </div>
        </>
        }
    }
}

impl HomePage {
    pub fn view_users(&self, ctx: &Context<Self>) -> Html {
        let mut widgets = Vec::new();
        for usr in ctx.props().users.iter() {
            let widget = self.view_user_widget(usr, ctx);
            widgets.push(widget);
        }
        html! {
            { for widgets.into_iter().by_ref() }
        }
    }

    pub fn view_user_widget(&self, user: &UserMeta, ctx: &Context<Self>) -> Html {
        if let UserMeta::Mint(_) = user {
            return html! {};
        }
        let mut id = "".into();
        match user {
            UserMeta::Owner(_) => id = "owner".into(),
            UserMeta::User(usr) => {
                id = format!("user-{}", usr.name);
            }
            _ => {}
        }
        html! {
            <div class="box columns mt-1 card-border" >

                <div class="column is-1 text-center">
                    <figure class="image is-64x64" >
                        <img class="full-cover"  src={user.get_avatar()} alt="user's avatar" />
                    </figure>
                    { self.get_user_tag(user) }
                </div>

                <div class="column" style="max-width:768px">
                    <div class="level " style="margin-bottom:0">
                        <div class="level-left">
                            <div class="level-item" id={id}>
                                <span class="title is-5">{ user.get_name() }</span>
                            </div>
                        </div>
                        <div class="level-right">
                            <div class="level-item">
                                <span class="pr=2">{ "💰\t" }</span>
                                <span class="balance">{ user.get_balance() }</span>
                            </div>
                        </div>
                    </div>

                    <hr class="doted"/>

                    <div class="column no-padding" >
                        <p class="auto-line-break"> { "Public Key: " }
                            <code>{ user.get_public_key() }</code>
                        </p>
                        </div>

                    <div class="column no-padding" >
                        <p class="auto-line-break">{ "Secret Key: " }
                            <code>{ user.get_secret_key() }</code>
                        </p>
                    </div>

                <div class="column no-padding">
                {
                    self.view_transactions(user, ctx)
                }
                </div>
            </div>

        </div>
        }
    }

    fn get_user_tag(&self, user: &UserMeta) -> Html {
        match user {
            UserMeta::Owner(_) => html! {
                <span class="tag is-danger">
                    {"Owner"}
                </span>
            },
            UserMeta::User(_) => html! {
                <span class="tag is-dark">
                    {"User"}
                </span>
            },
            _ => html! {},
        }
    }

    fn view_transactions(&self, usr: &UserMeta, ctx: &Context<Self>) -> Html {
        html! {
            <table class="table is-fullwidth is-hoverable">
                <thead>
                    <tr>
                        <th>{ "From" }</th>
                        <th>{ "To" }</th>
                        <th>{ "Amount" }</th>
                        <th>{ "Tips" }</th>
                    </tr>
                </thead>
                <tbody>
                {
                    self.view_transactions_row(usr, ctx)
                }
                </tbody>
            </table>
        }
    }

    fn view_transactions_row(&self, usr: &UserMeta, ctx: &Context<Self>) -> Html {
        let mut widgets = Vec::new();
        match usr {
            UserMeta::Owner(ref owner) => {
                let pubkey = &owner.public_key;
                owner.chain.chain.iter().for_each(|block| {
                    block.data.iter().for_each(|trans| {
                        if &trans.from == pubkey {
                            let (name, is_mint, is_owner) = self.get_user_name(&trans.to, ctx);
                            let href = format!("#user-{}", name.replace(" ", "-"));
                            let td = if is_mint {
                                html! {
                                        <td>{ name }</td>
                                }
                            } else if is_owner {
                                html! {
                                        <td><a href="#owner" >{ name }</a></td>
                                }
                            } else {
                                html! {
                                        <td><a href={href} >{ name }</a></td>
                                }
                            };
                            widgets.push(html! {
                                <tr>
                                    <td><a href="#owner">{ owner.name.clone() }</a></td>
                                    { td }
                                    <td>{ format!("{}", trans.amount) }</td>
                                    <td>{ format!("{}", trans.tips) }</td>
                                </tr>
                            });
                        } else if &trans.to == pubkey {
                            let (name, is_mint, is_owner) = self.get_user_name(&trans.from, ctx);
                            let href = format!("#user-{}", name.replace(" ", "-"));
                            let td = if is_mint {
                                html! {
                                        <td>{ name }</td>
                                }
                            } else if is_owner {
                                html! {
                                        <td><a href="#owner" >{ name }</a></td>
                                }
                            } else {
                                html! {
                                        <td><a href={href} >{ name }</a></td>
                                }
                            };
                            widgets.push(html! {
                                <tr>
                                    { td }
                                    <td><a href="#owner">{  owner.name.clone()  }</a></td>
                                    <td>{ format!("{}", trans.amount) }</td>
                                    <td>{ format!("{}", trans.tips) }</td>
                                </tr>
                            });
                        }
                    });
                });
            }
            UserMeta::User(ref user) => {
                let pubkey = &user.public_key;
                user.chain.chain.iter().for_each(|block| {
                    block.data.iter().for_each(|trans| {
                        if &trans.from == pubkey {
                            let (name, is_mint, is_owner) = self.get_user_name(&trans.to, ctx);
                            let href = format!("#user-{}", name.replace(" ", "-"));
                            let href1 = format!("#user-{}", user.name.replace(" ", "-"));
                            let td = if is_mint {
                                html! {
                                        <td>{ name }</td>
                                }
                            } else if is_owner {
                                html! {
                                        <td><a href="#owner" >{ name }</a></td>
                                }
                            } else {
                                html! {
                                        <td><a href={href} >{ name }</a></td>
                                }
                            };
                            widgets.push(html! {
                                <tr>
                                    <td><a href={href1}>{ user.name.clone() }</a></td>
                                    { td }
                                    <td>{ format!("{}", trans.amount) }</td>
                                    <td>{ format!("{}", trans.tips) }</td>
                                </tr>
                            });
                        } else if &trans.to == pubkey {
                            let (name, is_mint, is_owner) = self.get_user_name(&trans.from, ctx);
                            let href = format!("#user-{}", name.replace(" ", "-"));
                            let href1 = format!("#user-{}", user.name.replace(" ", "-"));
                            let td = if is_mint {
                                html! {
                                    <td>{ name }</td>
                                }
                            } else if is_owner {
                                html! {
                                    <td><a href="#owner" >{ name }</a></td>
                                }
                            } else {
                                html! {
                                    <td><a href={href}>{ name }</a></td>
                                }
                            };
                            widgets.push(html! {
                                <tr>
                                    { td }
                                    <td><a href={href1}>{  user.name.clone()  }</a></td>
                                    <td>{ format!("{}", trans.amount) }</td>
                                    <td>{ format!("{}", trans.tips) }</td>
                                </tr>
                            });
                        }
                    });
                });
            }
            _ => {}
        }
        html! {{  for widgets.into_iter().by_ref() } }
    }

    fn get_user_name(&self, pubkey: &PubKey, ctx: &Context<Self>) -> (String, bool, bool) {
        let mut name = "".into();
        let mut is_mint = false;
        let mut is_owner = false;
        let users = ctx.props().users.as_ref();
        for usr in users.iter() {
            match usr {
                UserMeta::Mint(mint) => {
                    if &mint.public_key == pubkey {
                        name = usr.get_name();
                        is_mint = true;
                        break;
                    }
                }
                UserMeta::Owner(owner) => {
                    if &owner.public_key == pubkey {
                        name = usr.get_name();
                        is_owner = true;
                        break;
                    }
                }
                UserMeta::User(user) => {
                    if &user.public_key == pubkey {
                        name = usr.get_name();
                        break;
                    }
                }
            }
        }
        (name, is_mint, is_owner)
    }

    fn get_signature(&self) -> String {
        match self.transaction {
            None => "".into(),
            Some(ref trans) => trans.0.signature.to_hex(),
        }
    }

    fn show_signature_controler(&self) {
        log::trace!("turn on/off signature");
        if self.refs.sign_passed {
            if let Some(ele) = self.refs.node_ref_signature.cast::<HtmlTextAreaElement>() {
                ele.set_class_name("textarea is-small");
                ele.set_value(&self.get_signature());
            }
            if let Some(ele) = self.refs.node_ref_label_signature.cast::<HtmlElement>() {
                ele.set_class_name("");
            }
            return;
        }
        if let Some(ele) = self.refs.node_ref_signature.cast::<HtmlTextAreaElement>() {
            ele.set_class_name("textarea is-small no-display");
            ele.set_value("");
        }
        if let Some(ele) = self.refs.node_ref_label_signature.cast::<HtmlElement>() {
            ele.set_class_name("no-display");
        }
    }
}

impl Refs {
    fn clear_help_message(&self, typ: &str) {
        if typ == "user" {
            if let Some(ele) = self.node_ref_help_user.cast::<HtmlElement>() {
                ele.set_class_name("help no-display");
            }
        } else if typ == "transfer" {
            if let Some(ele) = self.node_ref_help_from.cast::<HtmlElement>() {
                ele.set_class_name("help no-display");
            }
            if let Some(ele) = self.node_ref_help_to.cast::<HtmlElement>() {
                ele.set_class_name("help no-display");
            }
            if let Some(ele) = self.node_ref_help_amount.cast::<HtmlElement>() {
                ele.set_class_name("help no-display");
            }
            if let Some(ele) = self.node_ref_help_tips.cast::<HtmlElement>() {
                ele.set_class_name("help no-display");
            }
        }
    }

    fn clear_input_content(&self) {
        if let Some(ele) = self.node_ref_from.cast::<HtmlInputElement>() {
            ele.set_value("");
        }
        if let Some(ele) = self.node_ref_to.cast::<HtmlInputElement>() {
            ele.set_value("");
        }
        if let Some(ele) = self.node_ref_amount.cast::<HtmlInputElement>() {
            ele.set_value_as_number(0.0);
        }
        if let Some(ele) = self.node_ref_tips.cast::<HtmlInputElement>() {
            ele.set_value_as_number(0.0);
        }
        if let Some(ele) = self.node_ref_secret.cast::<HtmlInputElement>() {
            ele.set_value("");
        }
    }

    fn rm_notification(&self) -> Callback<MouseEvent> {
        let no_user = self.node_ref_no_user.clone();
        Callback::from(move |_| {
            if let Some(ele) = no_user.cast::<HtmlElement>() {
                ele.set_class_name("notification no-display is-light");
            }
        })
    }
}
