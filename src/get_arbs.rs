extern crate serde_json;

use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Ticker {
    pair: String,
    volume: f64,
}

#[derive(Debug)]
pub struct Pair {
    name: String,
    volume: f64,
}

#[derive(Debug)]
pub struct ArbPair {
    usdt: Pair,
    btc: Pair,
    arb_symbol: String,
}

#[tokio::main]
pub async fn get_arbs() -> Result<HashMap<String,ArbPair>, reqwest::Error> {
    let btc_price_res = reqwest::get("https://api.binance.com/api/v3/avgPrice?symbol=BTCUSDT").await?;
    let btc_price_json: Value = serde_json::from_str(&btc_price_res.text().await?).unwrap();
    let btc_price: f64 = btc_price_json.get("price").unwrap().as_str().unwrap().parse().unwrap();
    // println!("{:?}", btc_price);


    let ticker_res = reqwest::get("https://api.binance.com/api/v3/ticker/24hr").await?;
    let ticker_json: Vec<Value> = serde_json::from_str(&ticker_res.text().await?).unwrap();
    let mut tickers: Vec<Ticker> = Vec::new();
    for item in &ticker_json {
        tickers.push(
            Ticker {
                pair: String::from(item.get("symbol").unwrap().as_str().unwrap()),
                volume: item.get("quoteVolume").unwrap().as_str().unwrap().parse().unwrap(),
            }
        );

    }
    // println!("{:?}", tickers);


    let exchange_info_res = reqwest::get("https://api.binance.com/api/v3/exchangeInfo").await?;
    let exchange_info_serde: Value = serde_json::from_str(&exchange_info_res.text().await?).unwrap();
    let exchange_info_json: &Vec<Value> = exchange_info_serde.get("symbols").unwrap().as_array().unwrap();
    // println!("{:?}", exchange_info[0]);
    let mut exchange_info: Vec<&str> = Vec::new();
    for item in exchange_info_json {
        if item.get("isSpotTradingAllowed").unwrap().as_bool().unwrap()
            && !is_white_list(item.get("symbol").unwrap().as_str().unwrap())
        {
            exchange_info.push(item.get("symbol").unwrap().as_str().unwrap());
        }
    }
    exchange_info.sort();
    // println!("{:?}", exchange_info);


    let mut arbs: Vec<&str> = Vec::new();
    for x in exchange_info {
        if &x[x.len() -3..x.len()] == "BTC" {
            arbs.push(&x[0 .. x.len() - 3])
        } else if &x[x.len() -4..x.len()] == "USDT" {
            arbs.push(&x[0..x.len() - 4])
        }
    }
    arbs.sort();
    // println!("{:?}", arbs);

    let book_hash = arbs.windows(2).filter_map(|s| {
        if s[0] == s[1] {
            let arb = s[0];
            let mut pair = (None, None);
            for ticker in &tickers {
                if ticker.pair == arb.to_string() + "USDT" && ticker.volume > 500_000. {
                    pair.0 = Some(Pair { name: arb.to_string() + "USDT", volume: ticker.volume });
                } else if ticker.pair == arb.to_string() + "BTC" && ticker.volume * btc_price > 500_000. {
                    pair.1 = Some(Pair { name: arb.to_string() + "BTC", volume: (ticker.volume * btc_price) });
                }
            }
            if let (Some(usdt), Some(btc)) = pair {
                return Some((arb.to_string(), ArbPair { usdt, btc, arb_symbol: arb.to_string() }));
            }
        }
        None
    }).collect::<HashMap<_,_>>();
    // println!("{:#?}", book_hash.len());
    Ok(book_hash)
}


fn is_white_list(pair: &str) -> bool {
    for x in ["BEAR", "BULL", "DOWN", "UP"].iter() {
        if pair.contains(x) {
            return true;
        }
    }
    return false;
}