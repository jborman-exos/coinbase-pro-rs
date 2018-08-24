extern crate serde;
extern crate tokio;
extern crate base64;
extern crate hmac;
extern crate sha2;

use std::fmt::Debug;
use hyper::{HeaderMap, Request, Body, Uri, Method};
use hyper::header::HeaderValue;
use private::hmac::{Hmac, Mac};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use serde_json::{self, Value};

use super::Result;
use structs::private::*;
use structs::reqs;

use public::Public;

pub struct Private {
    _pub: Public,
    key: String,
    secret: String,
    passphrase: String
}

impl Private {
    pub fn get_sync<U>(&self, uri: &str) -> Result<U>
        where U: Debug + Send + 'static,
              U: for<'de> serde::Deserialize<'de>
    {
        self._pub.get_sync_with_req(self.request(Method::GET, uri, "".to_string()))
    }

    pub fn post_sync<U>(&self, uri: &str, order: reqs::Order) -> Result<U>
        where U: Debug + Send + 'static,
              U: for<'de> serde::Deserialize<'de>
    {
        let body_str = serde_json::to_string(&order)
            .expect("cannot to_string post body");
        println!("DEBUG1: {}", body_str);
        self._pub.get_sync_with_req(self.request(Method::POST, uri, body_str))
    }

    fn  sign(&self, timestamp: u64, method: Method, uri: &str, body_str: &str) -> String {
        let key = base64::decode(&self.secret).expect("base64::decode secret");
        let mut mac: Hmac<sha2::Sha256> = Hmac::new_varkey(&key).expect("Hmac::new(key)");
        mac.input((timestamp.to_string()+method.as_str()+uri+body_str).as_bytes());
        base64::encode(&mac.result().code())
    }

//   from python
//POST /orders HTTP/1.1
//Host: localhost:3000
//User-Agent: python-requests/2.13.0
//Accept-Encoding: gzip, deflate
//Accept: */*
//Connection: keep-alive
//Content-Length: 92
//Content-Type: Application/JSON
//CB-ACCESS-SIGN: Hy8vbkj3r/XoaT46oQveZs8OIl6zX/xRR6lKTSvfxuk=
//CB-ACCESS-TIMESTAMP: 1535003621.005189
//CB-ACCESS-KEY: 1d0dc0f7b4e808d430b95d8fed7df3ea
//CB-ACCESS-PASSPHRASE: sandbox
//
//{"product_id": "BTC-USD", "side": "buy", "type": "limit", "price": "100.00", "size": "0.01"}

    fn request(&self, method: Method, _uri: &str, body_str: String) -> Request<Body> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("leap-second").as_secs();

        let uri: Uri = (self._pub.uri.to_string() + _uri).parse().unwrap();

        let mut req = Request::builder();
        req.method(&method);
        req.uri(uri);

        let sign = self.sign(timestamp, method, _uri, &body_str);

        req.header("User-Agent", Public::USER_AGENT);
        req.header("Content-Type", "Application/JSON");
//        req.header("Accept", "*/*");
        req.header("CB-ACCESS-KEY", HeaderValue::from_str(&self.key).unwrap());
        req.header("CB-ACCESS-SIGN", HeaderValue::from_str(&sign).unwrap());
        req.header("CB-ACCESS-TIMESTAMP", HeaderValue::from_str(&timestamp.to_string()).unwrap());
        req.header("CB-ACCESS-PASSPHRASE", HeaderValue::from_str(&self.passphrase).unwrap());

        req.body(body_str.into()).unwrap()
    }

    pub fn new(key: &str, secret: &str, passphrase: &str) -> Self {
        Self {
            _pub: Public::new(),
            key: key.to_string(),
            secret: secret.to_string(),
            passphrase: passphrase.to_string()
        }
    }

    pub fn get_accounts(&self) -> Result<Vec<Account>> {
        self.get_sync("/accounts")
    }

    pub fn get_account(&self, id: Uuid) -> Result<Account> {
        self.get_sync(&format!("/accounts/{}", id))
    }

    pub fn get_account_hist(&self, id: Uuid) -> Result<Vec<AccountHistory>> {
        self.get_sync(&format!("/accounts/{}/ledger", id))
            .map(|xs: Vec<AccountHistory>| xs.into_iter()
                 .map(|x| AccountHistory{_type: (&x.details).into(), ..x})
                    .collect())
    }

    pub fn get_account_holds(&self, id: Uuid) -> Result<Vec<AccountHolds>> {
        self.get_sync(&format!("/accounts/{}/holds", id))
    }

    fn set_order(&self, order: reqs::Order) -> Result<Order> {
        self.post_sync(&format!("/orders"), order)
    }

    pub fn buy_limit(&self, product_id: &str, size: f64, price: f64, post_only: bool
                     , time_in_force: Option<reqs::OrderTimeInForce>) -> Result<Order> {
        self.set_order(reqs::Order::limit(product_id
            , reqs::OrderSide::Buy, size, price, post_only, time_in_force))
    }

    pub fn sell_limit(&self, product_id: &str, size: f64, price: f64, post_only: bool
                      , time_in_force: Option<reqs::OrderTimeInForce>) -> Result<Order> {
        self.set_order(reqs::Order::limit(product_id
            , reqs::OrderSide::Sell, size, price, post_only, time_in_force))
    }

    pub fn buy_market(&self, product_id: &str, size: f64) -> Result<Order> {
        self.set_order(reqs::Order::market(product_id
            , reqs::OrderSide::Buy, size))
    }

    pub fn sell_market(&self, product_id: &str, size: f64) -> Result<Order> {
        self.set_order(reqs::Order::market(product_id
            , reqs::OrderSide::Sell, size))
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    static KEY: &str = "1d0dc0f7b4e808d430b95d8fed7df3ea";
    static PASSPHRASE: &str = "sandbox";
    static SECRET: &str = "dTUic8DZPqkS77vxhJFEX5IBr13FcFHTzWYOARgT9kDWGdN03uvxBbH/hVy8f4O5RDmuf+9wNpEfhYhw2FCWyA==";

    #[test]
    fn test_get_accounts() {
        let client = Private::new(KEY, SECRET, PASSPHRASE);
        let accounts = client.get_accounts().unwrap();
        assert!(format!("{:?}", accounts)
            .contains(r#"currency: "BCH", balance: 0.0, available: 0.0, hold: 0.0, profile_id: "#));
        assert!(format!("{:?}", accounts)
            .contains(r#"currency: "ETH", balance: 0.0, available: 0.0, hold: 0.0, profile_id: "#));
    }

    #[test]
    fn test_get_account() {
//        super::super::pretty_env_logger::init_custom_env("RUST_LOG=trace");
        let client = Private::new(KEY, SECRET, PASSPHRASE);
        let coin_acc = client.get_accounts().unwrap().into_iter().find(|x| x.currency == "BTC").unwrap();
        let account = client.get_account(coin_acc.id);
        let account_str = format!("{:?}", account);
        assert!(account_str.contains("id:"));
        assert!(account_str.contains("currency: \"BTC\""));
        assert!(account_str.contains("balance:"));
        assert!(account_str.contains("available:"));
        assert!(account_str.contains("hold:"));
        assert!(account_str.contains("profile_id:"));
    }

    #[test]
    fn test_get_account_hist() {
//        super::super::pretty_env_logger::init_custom_env("RUST_LOG=trace");
        let client = Private::new(KEY, SECRET, PASSPHRASE);
        let coin_acc = client.get_accounts().unwrap().into_iter().find(|x| x.currency == "USD").unwrap();
        let account = client.get_account_hist(coin_acc.id);
        let account_str = format!("{:?}", account);
//        println!("{}", account_str);
        assert!(account_str.contains("type: Match, details: Match"));
    }

    #[test]
    #[ignore]
    fn test_get_account_holds() {
//        super::super::pretty_env_logger::init_custom_env("RUST_LOG=trace");
        let client = Private::new(KEY, SECRET, PASSPHRASE);
        let coin_acc = client.get_accounts().unwrap().into_iter().find(|x| x.currency == "USD").unwrap();
        let acc_holds = client.get_account_holds(coin_acc.id);
        let str = format!("{:?}", acc_holds);
//        assert!(account_str.contains("transfer_type: Deposit"));
        //println!("{:?}", str);
        assert!(false); // TODO: holds are empty now
    }

    #[test]
    fn test_new_order_ser() {
        let order = reqs::Order::market("BTC-UST", reqs::OrderSide::Buy, 1.1);
        let str = serde_json::to_string(&order).unwrap();
        assert_eq!(vec![0],
                   str.match_indices("{").map(|(x,_)| x).collect::<Vec<_>>());
    }

    #[test]
    fn test_set_order_limit() {
        let client = Private::new(KEY, SECRET, PASSPHRASE);
        let order = client.buy_limit("BTC-USD", 1.0, 1.12, true, None).unwrap();
        let str = format!("{:?}", order);
        assert!(str.contains("side: Buy"));
        assert!(str.contains("_type: Limit {"));
        let order = client.sell_limit("BTC-USD", 0.001, 100000.0, true, None).unwrap();
        let str = format!("{:?}", order);
        assert!(str.contains("side: Sell"));
        assert!(str.contains("_type: Limit {"));
    }

    #[test]
    fn test_set_order_limit_gtc() {
        let client = Private::new(KEY, SECRET, PASSPHRASE);
        let order = client.buy_limit("BTC-USD", 1.0, 1.12, true
            , Some(reqs::OrderTimeInForce::GTT {
                cancel_after: reqs::OrderTimeInForceCancelAfter::Min
            })).unwrap();
        let str = format!("{:?}", order);
        assert!(str.contains("time_in_force: GTT { expire_time: 2"));
    }

    #[test]
    fn test_set_order_market() {
        let client = Private::new(KEY, SECRET, PASSPHRASE);
        let order = client.buy_market("BTC-USD", 0.001).unwrap();
        let str = format!("{:?}", order);
        assert!(str.contains("side: Buy"));
        assert!(str.contains("_type: Market {"));
        let order = client.sell_market("BTC-USD", 0.001).unwrap();
        let str = format!("{:?}", order);
        assert!(str.contains("side: Sell"));
        assert!(str.contains("_type: Market {"));
    }
}



