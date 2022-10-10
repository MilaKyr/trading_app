use crate::actions::{ClientActions, ServerActions};
use crate::errors::Error;
use crate::products::Product;
use crate::storage::TransactionStorage;
use crate::trader::Transaction;
use std::collections::HashMap;
use std::sync::RwLock;
use tokio::sync::mpsc::Sender;

#[derive(Default)]
pub struct TransactionService {
    traders: RwLock<HashMap<u16, Sender<String>>>,
    sells: TransactionStorage,
    buys: TransactionStorage,
}

impl TransactionService {
    pub async fn inform_all(&self, product: Product) -> Result<(), Error> {
        let message = Self::inform_about_trade(product);
        for trader_send in self.get_all_trader_addrs() {
            trader_send.send(message.clone()).await?;
        }
        Ok(())
    }

    pub async fn confirm(&self, trader_id: u16, product: Product) -> Result<(), Error> {
        if let Some(trader_send) = self.get_trader_addr(trader_id) {
            let message = Self::ack_order(product);
            trader_send.send(message).await?;
        };
        Ok(())
    }

    pub async fn send_error(&self, trader_id: u16, error_msg: String) -> Result<(), Error> {
        if let Some(trader_send) = self.get_trader_addr(trader_id) {
            trader_send.send(error_msg).await?;
        };
        Ok(())
    }

    pub fn remove_trader(&self, trader_id: u16) {
        self.traders.write().unwrap().remove(&trader_id);
    }

    pub fn register_trader(&self, trader_id: u16, stream_addr: Sender<String>) {
        self.traders.write().unwrap().insert(trader_id, stream_addr);
    }

    pub fn register_order(&self, transaction: Transaction) {
        match transaction.action {
            ClientActions::Buy => self.buys.add(transaction.into()),
            ClientActions::Sell => self.sells.add(transaction.into()),
        };
    }

    pub fn try_trade_with(&self, transaction: Transaction) -> Option<Product> {
        match transaction.action {
            ClientActions::Buy => self.find_seller(transaction.product, transaction.trader_id),
            ClientActions::Sell => self.find_buyer(transaction.product, transaction.trader_id),
        }
    }

    pub fn log_trade(product: Product) -> String {
        format!("{} ({})", ServerActions::Trade, product)
    }

    fn inform_about_trade(product: Product) -> String {
        format!("{}:{}", ServerActions::Trade, product)
    }

    fn ack_order(product: Product) -> String {
        format!("{}:{}", ServerActions::Ack, product)
    }

    fn get_trader_addr(&self, trader_id: u16) -> Option<Sender<String>> {
        self.traders.write().unwrap().get(&trader_id).cloned()
    }

    fn get_all_trader_addrs(&self) -> Vec<Sender<String>> {
        let traders = self.traders.write().unwrap();
        traders.values().cloned().into_iter().collect()
    }

    fn find_seller(&self, product: Product, author_id: u16) -> Option<Product> {
        self.sells
            .try_find(product, author_id)
            .map(|(matched_product, position)| {
                self.sells.remove_at(position);
                matched_product
            })
    }

    fn find_buyer(&self, product: Product, author_id: u16) -> Option<Product> {
        self.buys
            .try_find(product, author_id)
            .map(|(matched_product, position)| {
                self.buys.remove_at(position);
                matched_product
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Transaction;

    #[test]
    fn test_register_buying_trader() {
        let tr_service = TransactionService::default();
        let transaction = Transaction {
            trader_id: 0,
            action: ClientActions::Buy,
            product: Product::Apple,
        };
        tr_service.register_order(transaction);
        let buys = tr_service.buys.data.read().unwrap();
        let sells = tr_service.sells.data.read().unwrap();
        assert_eq!(buys.len(), 1);
        assert!(buys.contains(&transaction.into()));
        assert_eq!(sells.len(), 0);
    }

    #[test]
    fn test_register_selling_trader() {
        let tr_service = TransactionService::default();
        let transaction = Transaction {
            trader_id: 0,
            action: ClientActions::Sell,
            product: Product::Apple,
        };
        tr_service.register_order(transaction);
        let buys = tr_service.buys.data.read().unwrap();
        let sells = tr_service.sells.data.read().unwrap();
        assert_eq!(sells.len(), 1);
        assert!(sells.contains(&transaction.into()));
        assert_eq!(buys.len(), 0);
    }

    #[test]
    fn test_try_trade_with_seller() {
        let tr_service = TransactionService::default();
        let buy_transaction = Transaction {
            trader_id: 1,
            action: ClientActions::Buy,
            product: Product::Apple,
        };
        tr_service.register_order(buy_transaction);
        let sell_transaction = Transaction {
            trader_id: 0,
            action: ClientActions::Sell,
            product: Product::Apple,
        };
        let result = tr_service.try_trade_with(sell_transaction);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Product::Apple);
    }

    #[test]
    fn test_try_trade_with_seller_faild() {
        let tr_service = TransactionService::default();
        let sell_transaction = Transaction {
            trader_id: 0,
            action: ClientActions::Sell,
            product: Product::Apple,
        };
        let result = tr_service.try_trade_with(sell_transaction);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_trade_with_buyer() {
        let tr_service = TransactionService::default();
        let sell_transaction = Transaction {
            trader_id: 0,
            action: ClientActions::Sell,
            product: Product::Apple,
        };
        tr_service.register_order(sell_transaction);
        let buy_transaction = Transaction {
            trader_id: 1,
            action: ClientActions::Buy,
            product: Product::Apple,
        };
        let result = tr_service.try_trade_with(buy_transaction);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Product::Apple);
    }

    #[test]
    fn test_try_trade_with_just_buyer() {
        let tr_service = TransactionService::default();
        let transaction = Transaction {
            trader_id: 1,
            action: ClientActions::Buy,
            product: Product::Apple,
        };
        tr_service.register_order(transaction);
        let result = tr_service.try_trade_with(transaction);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_trade_with_buyer_failed() {
        let tr_service = TransactionService::default();
        let transaction = Transaction {
            trader_id: 0,
            action: ClientActions::Buy,
            product: Product::Apple,
        };
        let result = tr_service.try_trade_with(transaction);
        assert!(result.is_none());
    }

    #[test]
    fn test_inform_about_trade() {
        let expected_result = "TRADE:APPLE".to_string();
        let result = TransactionService::inform_about_trade(Product::Apple);
        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_log_trade() {
        let expected_result = "TRADE (APPLE)".to_string();
        let result = TransactionService::log_trade(Product::Apple);
        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_ack_order() {
        let expected_result = "ACK:APPLE".to_string();
        let result = TransactionService::ack_order(Product::Apple);
        assert_eq!(expected_result, result)
    }
}
