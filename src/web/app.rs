use crate::web::event_bus::{EventBus, Request};
use crate::{block::*, blockchain::*, constant::*, secp256k1::*, transaction::*};
use yew::html::Scope;
use yew::prelude::*;
//use yew_agent::{Agent, AgentLink, Dispatched, Dispatcher};
use std::rc::Rc;
use yew_agent::{Bridge, Bridged};
use yew_router::prelude::*;

use crate::web::{pages::home::*, pages::not_found::*};

#[function_component(AppWrap)]
pub fn app_wrap() -> Html {
    html! {
            <BrowserRouter>
                <App />
            <main>
                <Switch<Route> render={Switch::render(switch)} />
            </main>
            </BrowserRouter>

    }
}

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: &Route) -> Html {
    match routes.clone() {
        Route::Home => {
            html! { <HomePage  users={Rc::new(vec![])} /> }
        }
        Route::NotFound => {
            html! { <PageNotFound /> }
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum UserMeta {
    User(User),
    Owner(Owner),
    Mint(Mint),
}
impl UserMeta {
    pub fn chain(&self) -> &BlockChain {
        match self {
            Self::Mint(mint) => &mint.chain,
            Self::Owner(owner) => &owner.chain,
            Self::User(user) => &user.chain,
        }
    }

    pub fn chain_len(&self) -> usize {
        match self {
            Self::Mint(mint) => mint.chain.len(),
            Self::Owner(owner) => owner.chain.len(),
            Self::User(user) => user.chain.len(),
        }
    }

    pub fn push(&mut self, trans: &Transaction) {
        let trans = trans.clone();
        match self {
            Self::Mint(mint) => mint.transactions.push(trans),
            Self::Owner(owner) => owner.transactions.push(trans),
            Self::User(user) => user.transactions.push(trans),
        }
    }

    // push block to user
    pub fn push_bloc(&mut self, bloc: Block) {
        match self {
            Self::Mint(mint) => mint.chain.chain.push(bloc),
            Self::Owner(owner) => owner.chain.chain.push(bloc),
            Self::User(user) => user.chain.chain.push(bloc),
        }
    }

    pub fn rm_trans_bloc(&mut self, bloc: &Block) {
        match self {
            Self::Mint(mint) => {
                mint.transactions.retain(|trans| !bloc.data.contains(trans));
            }
            Self::Owner(owner) => {
                owner
                    .transactions
                    .retain(|trans| !bloc.data.contains(trans));
            }
            Self::User(user) => {
                user.transactions.retain(|trans| !bloc.data.contains(trans));
            }
        }
    }

    pub fn pub_key(&self) -> &PubKey {
        match self {
            Self::Mint(mint) => &mint.public_key,
            Self::Owner(owner) => &owner.public_key,
            Self::User(user) => &user.public_key,
        }
    }
}

impl UserMeta {
    pub fn get_name(&self) -> String {
        match self {
            UserMeta::Mint(_) => "MINT".into(),
            UserMeta::Owner(owner) => owner.name.clone(),
            UserMeta::User(user) => user.name.clone(),
        }
    }

    pub fn get_avatar(&self) -> String {
        match self {
            UserMeta::Mint(_) => "".into(),
            UserMeta::Owner(owner) => owner.avatar.clone(),
            UserMeta::User(user) => user.avatar.clone(),
        }
    }

    pub fn get_public_key(&self) -> String {
        match self {
            UserMeta::Mint(_) => "".into(),
            UserMeta::Owner(owner) => owner.public_key.to_hex(),
            UserMeta::User(user) => user.public_key.to_hex(),
        }
    }

    pub fn get_secret_key(&self) -> String {
        match self {
            UserMeta::Mint(_) => "".into(),
            UserMeta::Owner(owner) => owner.secret_key.to_hex(),
            UserMeta::User(user) => user.secret_key.to_hex(),
        }
    }

    pub fn get_balance(&self) -> String {
        match self {
            UserMeta::Mint(_) => "".into(),
            UserMeta::Owner(owner) => {
                let pubkey = &owner.public_key;
                let balance = owner.chain.get_balance(pubkey);
                format!("{:.3}", balance)
            }
            UserMeta::User(user) => {
                let pubkey = &user.public_key;
                let balance = user.chain.get_balance(pubkey);
                format!("{:.3}", balance)
            }
        }
    }
}

pub struct App {
    pub users: Rc<Vec<UserMeta>>,
    _producer: Box<dyn Bridge<EventBus>>,
}

pub enum Msg {
    UserMintCreate,
    UserCreate(String),
    UserCreated,
    UserCreateFailed(String),
    TransferInitialUser(Transaction),
    Transfer(Transaction),
    MineTransaction(Transaction),
    MinedTransaction((Block, PubKey)),
    Transfered,
    TransferFailed,
    InvalidTransaction,
    InvalidUserOrTransaction,
}

#[derive(Clone, PartialEq)]
pub struct Mint {
    pub(crate) name: String,
    pub(crate) avatar: String,
    pub(crate) public_key: PubKey,
    pub(crate) secret_key: SecKey,
    pub(crate) balance: f64,
    pub(crate) transactions: Vec<Transaction>,
    pub(crate) chain: BlockChain,
}

#[derive(Clone, PartialEq)]
pub struct Owner {
    pub name: String,
    pub avatar: String,
    pub public_key: PubKey,
    pub secret_key: SecKey,
    pub balance: f64,
    pub transactions: Vec<Transaction>,
    pub chain: BlockChain,
}

#[derive(Clone, PartialEq)]
pub struct User {
    pub name: String,
    pub avatar: String,
    pub public_key: PubKey,
    pub balance: f64,
    pub(crate) secret_key: SecKey,
    pub transactions: Vec<Transaction>,
    pub chain: BlockChain,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let callback = move |e: Request| match e {
            Request::UserCreate(user) => Msg::UserCreate(user),
            Request::Transfer(trans) => Msg::Transfer(trans),
            Request::Transfered => Msg::Transfered,
        };
        ctx.link().send_message(Msg::UserMintCreate);
        Self {
            users: Rc::new(Vec::new()),
            //chain: BlockChain::new(),
            _producer: EventBus::bridge(ctx.link().callback(callback)),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::TransferInitialUser(trans) => {
                log::info!("transfer initial money to user: {}", trans.to.to_hex());
                Rc::make_mut(&mut self.users)
                    .iter_mut()
                    .for_each(|user| match user {
                        UserMeta::Owner(owner) => {
                            owner.transactions.push(trans.clone());
                        }
                        UserMeta::User(usr) => {
                            usr.transactions.push(trans.clone());
                        }
                        _ => {}
                    });
                let block_transactions = vec![trans];
                let block = Block::new(chrono::Utc::now(), block_transactions);
                self.sync(&block, None);
                ctx.link().send_message(Msg::Transfered);
                //self.add_block(block);
                //log::debug!("mine the transaction");
                //ctx.link().send_message(Msg::MineTransaction(trans));
            }
            Msg::Transfer(trans) => {
                assert_ne!(trans.signature, MINT_KEY.3, "transaction must be signed");
                log::trace!(
                    "dispatching to all users the received transaction from agent: {:?}",
                    trans
                );
                // TODO make sure the transaction is valid for sure
                let sender_chain = self.find(&trans.from);
                match sender_chain {
                    None => {
                        log::error!("user not found or invalid transaction!");
                        ctx.link().send_message(Msg::InvalidUserOrTransaction);
                        return true;
                    }
                    Some(chain) => {
                        log::debug!("first validating the transaction");
                        if !trans.is_valid(chain) {
                            // not valid
                            ctx.link().send_message(Msg::InvalidTransaction);
                            return true;
                        }
                    }
                }
                // FIXME use atomic to ensure that mining is cancelled after it's mined
                Rc::make_mut(&mut self.users)
                    .iter_mut()
                    .for_each(|user| match user {
                        /*
                         *UserMeta::Mint(mint) => {
                         *    mint.transactions.push(trans.clone());
                         *}
                         */
                        UserMeta::Owner(owner) => {
                            owner.transactions.push(trans.clone());
                        }
                        UserMeta::User(usr) => {
                            usr.transactions.push(trans.clone());
                        }
                        _ => {}
                    });
                log::debug!("mine the transaction");
                ctx.link().send_message(Msg::MineTransaction(trans));
            }
            Msg::InvalidTransaction => {
                log::info!("invalid transaction");
            }
            Msg::InvalidUserOrTransaction => {
                log::info!("invalid transaction or user not found");
            }
            Msg::MineTransaction(trans) => {
                self.random_mine(trans, ctx.link());
            }
            Msg::MinedTransaction((bloc, pub_key)) => {
                self.sync(&bloc, Some(&pub_key));
                ctx.link().send_message(Msg::Transfered);
            }
            Msg::Transfered => {
                // here update transaction records
                log::debug!("the transaction completed");
                return true;
            }
            Msg::UserMintCreate => {
                if self.users.is_empty() {
                    ctx.link().send_message(Msg::UserCreate("MINT".into()));
                }
            }
            Msg::UserCreate(user_name) => {
                if self.users.is_empty() {
                    log::info!("MINT USER CREATED");
                    let user = UserMeta::Mint(Mint {
                        name: "MINT".into(),
                        avatar: "assets/rust.png".into(),
                        balance: f64::NAN,
                        public_key: MINT_KEY.1,
                        secret_key: MINT_KEY.0,
                        transactions: Vec::new(),
                        chain: BlockChain::new(),
                    });
                    Rc::make_mut(&mut self.users).push(user);
                    return true;
                }
                log::trace!("received user name from agent: {}", user_name);
                let (secret_key, public_key) = Secp256K1::new().gen_keypair();
                let pubkey = public_key.clone();
                // FIXME change to == when MINT created
                let user = if self.users.len() == 1 {
                    log::info!("no user found, create Owner: {}", user_name);
                    let chain = self.users[0].chain().clone();
                    UserMeta::Owner(Owner {
                        name: user_name,
                        avatar: "assets/rust.png".into(),
                        balance: 0.0,
                        public_key,
                        secret_key,
                        transactions: Vec::new(),
                        chain,
                    })
                } else {
                    log::info!("create User: {}", user_name);
                    let chain = self.longest_chain();
                    UserMeta::User(User {
                        name: user_name,
                        avatar: "assets/rust-user.png".into(),
                        balance: 0.0,
                        public_key,
                        secret_key,
                        transactions: Vec::new(),
                        chain,
                    })
                };
                ctx.link().send_message(Msg::UserCreated);
                Rc::make_mut(&mut self.users).push(user);
                let mint = self.find_mint();
                assert!(mint.is_some(), "MINT not found");
                let mint = mint.unwrap();
                assert_ne!(&mint.public_key, &pubkey, "MINT equals");
                let trans = Transaction::new(mint.public_key, pubkey, 10000.0, None);
                ctx.link().send_message(Msg::TransferInitialUser(trans));
                return true;
            }
            Msg::UserCreated => {
                // FIXME notify user that it is created
                return false;
            }
            Msg::UserCreateFailed(_s) => {
                // FIXME notify user that it is not created
                return false;
            }
            _ => {}
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        html! {
            <>
                { self.view_nav(link) }
                <HomePage users={self.users.clone()} />
            </>
        }
    }
}

impl App {
    fn view_nav(&self, _ctx: &Scope<Self>) -> Html {
        let mut name = "".into();
        let mut avatar = "".into();
        for usr in self.users.iter() {
            if let UserMeta::Owner(owner) = usr {
                name = owner.name.clone();
                avatar = owner.avatar.clone();
                break;
            }
        }
        html! {
                    <nav class="navbar" role="navigation" aria-label="main navigation">
                        <div class="navbar-brand">
                            <a class="navbar-item" href="#">
                                <h1 class="navbar-item is-size-3"> {"Black Bloc"} </h1
        >
                            </a>
                            <a class="navbar-item" href="#">
                                <figure class="image is-rounded pr-3">
                                    <img src="assets/rust.png" class="image"/>
                                </figure>
                            </a>
                        </div>
                        <div class="navbar-end">
                            <div class="navbar-item" >
                                <div class="field is-grouped">
                                    <a  href="#">
                                        <figure class="image is-rounded pr-3">
                                            <img style="width:auto;" src={ avatar } />
                                        </figure>
                                    </a>
                                    <div class="navbar-item has-dropdown is-hoverable">
                                    <a class="title is-5"  href="#">
                                        { name }
                                    </a>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </nav>
                }
    }

    // FIXME implement this later
    // make sure that all user's blockchain is synchronours
    // just compare all block's hash are the same
    pub fn synced(&self) -> bool {
        //use bitcoin_hashes::{sha256::Hash as Sha256, Hash};
        false
    }

    // FIXME implement this later atomic operation are required to make sure
    // that data-race memory-safety are guaranteed
    // if one has mined the transaction and seal them into block,
    // then push it to all the users
    pub fn sync(&mut self, bloc: &Block, pubkey: Option<&PubKey>) {
        Rc::make_mut(&mut self.users).iter_mut().for_each(|e| {
            if Some(e.pub_key()) != pubkey {
                e.push_bloc(bloc.clone());
                e.rm_trans_bloc(bloc);
            }
        });
    }

    // obtain the longest blockchain from all users
    pub fn longest_chain(&self) -> BlockChain {
        let mut ind = 0;
        let mut len = 0;
        for i in 0..self.users.len() {
            let usr = &self.users[i];
            if usr.chain_len() > len {
                ind = i;
                len = usr.chain_len();
            }
        }
        self.users[ind].chain().clone()
    }

    // randomly select a user to mine a transaction
    pub fn random_mine(&mut self, trans: Transaction, link: &Scope<Self>) {
        let now = chrono::Utc::now().timestamp_nanos();
        let mut ind = now as usize % self.users.len();
        if let UserMeta::Mint(_) = self.users[ind] {
            ind += 1;
            ind = ind % self.users.len();
        }
        match Rc::make_mut(&mut self.users)[ind] {
            UserMeta::Owner(ref mut owner) => {
                log::info!(
                    "{} mining the transaction: {}-{}",
                    owner.name,
                    trans.from.to_hex(),
                    trans.to.to_hex(),
                );
                let pub_key = &owner.public_key;
                owner.chain.add_transaction(trans);
                let bloc = owner.chain.mine_transaction(pub_key);
                link.send_message(Msg::MinedTransaction((bloc, pub_key.clone())));
            }
            UserMeta::User(ref mut user) => {
                log::info!(
                    "{} mining the transaction: {}-{}",
                    user.name,
                    trans.from.to_hex(),
                    trans.to.to_hex(),
                );
                let pub_key = &user.public_key;
                user.chain.add_transaction(trans);
                let bloc = user.chain.mine_transaction(pub_key);
                link.send_message(Msg::MinedTransaction((bloc, pub_key.clone())));
            }
            _ => {}
        }
    }

    pub fn find(&self, pubkey: &PubKey) -> Option<&BlockChain> {
        for user in self.users.iter() {
            match user {
                /*
                 *UserMeta::Mint(_) => {
                 *    if &mint.public_key == pubkey {
                 *        return Some(&mint.chain);
                 *    }
                 *    return None;
                 *}
                 */
                UserMeta::Owner(owner) => {
                    if &owner.public_key == pubkey {
                        return Some(&owner.chain);
                    }
                }
                UserMeta::User(usr) => {
                    if &usr.public_key == pubkey {
                        return Some(&usr.chain);
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn find_mint(&self) -> Option<&Mint> {
        let users = self.users.as_ref();
        let mut i = 0;
        for ind in 0..users.len() {
            if let UserMeta::Mint(_) = users[ind] {
                i = ind;
                break;
            }
        }
        match self.users.as_ref()[i] {
            UserMeta::Mint(ref mint) => Some(mint),
            _ => None,
        }
    }
}

pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    yew::start_app::<App>();
}
