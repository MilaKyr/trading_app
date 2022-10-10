use crate::actions::ClientActions;
use crate::errors::ClientError;
use crate::products::Product;
use crate::utils::split_at_colon;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio_util::codec::{Framed, LinesCodec};

pub struct Trader {
    pub trader_id: u16,
    pub lines: Framed<TcpStream, LinesCodec>,
    pub receiver_ch: Receiver<String>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Transaction {
    pub trader_id: u16,
    pub action: ClientActions,
    pub product: Product,
}

impl Transaction {
    pub fn new_from(trader_id: u16, message: String) -> Result<Transaction, ClientError> {
        let (action, product) =
            split_at_colon(&*message).ok_or(ClientError::InvalidTransactionMessage)?;
        let action = ClientActions::from_str(&*action.to_uppercase())?;
        let product = Product::from_str(&*product.to_uppercase())?;
        Ok(Self {
            trader_id,
            action,
            product,
        })
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "new {} order ('{}', {})",
            self.action.to_string().to_lowercase(),
            self.trader_id,
            self.product
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_from_str() {
        let trader_id = 0;
        let buy_order = "buy:onion".to_string();
        let expected_result = Transaction {
            trader_id,
            action: ClientActions::Buy,
            product: Product::Onion,
        };
        match Transaction::new_from(trader_id, buy_order) {
            Ok(result) => assert_eq!(result, expected_result),
            Err(_) => assert!(false),
        }
    }
    #[test]
    fn test_transaction_incorrect_message() {
        let buy_order = "buy onion".to_string();
        let result = Transaction::new_from(0, buy_order);
        assert!(matches!(
            result,
            Err(ClientError::InvalidTransactionMessage)
        ));
    }

    #[test]
    fn test_transaction_incorrect_product() {
        let buy_order = "buy:GME".to_string();
        let result = Transaction::new_from(0, buy_order);
        assert!(matches!(result, Err(ClientError::UnknownProduct)));
    }

    #[test]
    fn test_transaction_incorrect_action() {
        let buy_order = "buyy:APPLE".to_string();
        let result = Transaction::new_from(0, buy_order);
        assert!(matches!(result, Err(ClientError::UnknownAction)));
    }
}
