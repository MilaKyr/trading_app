mod actions;
mod consts;
mod errors;
mod products;
mod storage;
mod trader;
mod transaction_service;
mod utils;

use consts::{BUFFER_SIZE, LOCALHOST, PORT};
use errors::Error;
use futures::sink::SinkExt;
use log::{error, info};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime;
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};
use trader::{Trader, Transaction};
use transaction_service::TransactionService;
use utils::{get_greeting_message, init_logs};

fn main() -> Result<(), Error> {
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    init_logs();
    let transaction_service = Arc::new(TransactionService::default());
    rt.block_on(run_trading(transaction_service))
}

async fn run_trading(transaction_service: Arc<TransactionService>) -> Result<(), Error> {
    let address = format!("{}:{}", LOCALHOST, PORT);
    let listener = TcpListener::bind(address).await?;
    loop {
        let (stream, socket_addr) = listener.accept().await?;
        let trader_id = socket_addr.port();
        let transaction_service = Arc::clone(&transaction_service);
        info!("{}", get_greeting_message(socket_addr.ip(), trader_id)?);
        tokio::task::spawn(async move {
            if let Err(e) = process(trader_id, stream, transaction_service).await {
                error!("Error occurred! {}", e.to_string());
            }
        });
    }
}

fn handle_new_trader(
    trader_id: u16,
    stream: TcpStream,
    transaction_service: Arc<TransactionService>,
) -> Trader {
    let lines = Framed::new(stream, LinesCodec::new());
    let (sender, receiver) = channel(BUFFER_SIZE);
    transaction_service.register_trader(trader_id, sender);
    Trader {
        trader_id,
        lines,
        receiver_ch: receiver,
    }
}

async fn process(
    trader_id: u16,
    stream: TcpStream,
    transaction_service: Arc<TransactionService>,
) -> Result<(), Error> {
    let mut trader = handle_new_trader(trader_id, stream, transaction_service.clone());
    loop {
        tokio::select! {
            Some(msg) = trader.receiver_ch.recv() => {
                trader.lines.send(&msg).await?;
            }
            result = trader.lines.next() => match result {
                Some(Ok(line)) => read_transaction_message(trader_id, line, transaction_service.clone()).await?,
                Some(Err(e)) => error!("Error occurred while processing transaction. {}",e.to_string()),
                None => break,
            },
        }
    }
    transaction_service.remove_trader(trader_id);
    Ok(())
}

async fn read_transaction_message(
    trader_id: u16,
    line: String,
    transaction_service: Arc<TransactionService>,
) -> Result<(), Error> {
    match Transaction::new_from(trader_id, line) {
        Ok(transaction) => {
            info!("{}", transaction);
            transaction_service
                .confirm(transaction.trader_id, transaction.product)
                .await?;
            match transaction_service.try_trade_with(transaction) {
                Some(matched_item) => {
                    info!("{}", TransactionService::log_trade(matched_item));
                    transaction_service.inform_all(matched_item).await?;
                }
                None => transaction_service.register_order(transaction),
            }
        }
        Err(e) => {
            let error_msg = format!("{}", e.to_string());
            transaction_service.send_error(trader_id, error_msg).await?;
        }
    };
    Ok(())
}
