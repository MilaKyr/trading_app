use crate::products::Product;
use crate::Transaction;
use std::sync::RwLock;

type Position = usize;
type ProductInfo = (Product, Position);

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TransactionInfo {
    trader_id: u16,
    product: Product,
}

impl From<Transaction> for TransactionInfo {
    fn from(transaction: Transaction) -> Self {
        Self {
            trader_id: transaction.trader_id,
            product: transaction.product,
        }
    }
}

#[derive(Default, Debug)]
pub struct TransactionStorage {
    pub data: RwLock<Vec<TransactionInfo>>,
}

impl TransactionStorage {
    pub fn add(&self, transaction_info: TransactionInfo) {
        self.data.write().unwrap().push(transaction_info);
    }

    pub fn try_find(&self, item: Product, author_id: u16) -> Option<ProductInfo> {
        let transaction_info = TransactionInfo {
            trader_id: author_id,
            product: item,
        };
        let data = self.data.read().unwrap();
        data.iter()
            .position(|tr_info| same_product_diff_traders(tr_info, &transaction_info))
            .map(|position| {
                let product = data[position].product;
                (product, position)
            })
    }

    pub fn remove_at(&self, position: usize) {
        self.data.write().unwrap().remove(position);
    }
}

fn same_product_diff_traders(
    transaction_info_a: &TransactionInfo,
    transaction_info_b: &TransactionInfo,
) -> bool {
    transaction_info_a.product == transaction_info_b.product
        && transaction_info_a.trader_id != transaction_info_b.trader_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adding_to_transaction_storage() {
        let storage = TransactionStorage::default();
        let transaction_info = TransactionInfo {
            trader_id: 42,
            product: Product::Apple,
        };
        storage.add(transaction_info);
        let data = storage.data.read().unwrap();
        assert_eq!(data.len(), 1);
        assert!(data.contains(&transaction_info));
    }

    #[test]
    fn test_removing_at() {
        let storage = TransactionStorage::default();
        let transaction_info = TransactionInfo {
            trader_id: 42,
            product: Product::Apple,
        };
        storage.add(transaction_info);
        storage.remove_at(0);
        let data = storage.data.read().unwrap();
        assert_eq!(data.len(), 0);
        assert!(!data.contains(&transaction_info));
    }

    #[test]
    fn test_try_find_some() {
        let product = Product::Onion;
        let trader_id = 40;
        let storage = TransactionStorage::default();
        for transaction_info in [
            TransactionInfo {
                trader_id,
                product: Product::Apple,
            },
            TransactionInfo {
                trader_id: 42,
                product,
            },
        ] {
            storage.add(transaction_info)
        }
        match storage.try_find(product, trader_id) {
            Some((matched_product, pos)) => {
                assert_eq!(matched_product, product);
                assert_eq!(pos, 1);
            }
            None => assert!(false),
        }
    }

    #[test]
    fn test_try_find_first() {
        let product = Product::Onion;
        let trader_id = 40;
        let storage = TransactionStorage::default();
        for transaction_info in [
            TransactionInfo {
                trader_id,
                product: Product::Apple,
            },
            TransactionInfo {
                trader_id: 42,
                product,
            },
            TransactionInfo {
                trader_id: 43,
                product,
            },
        ] {
            storage.add(transaction_info)
        }
        match storage.try_find(product, trader_id) {
            Some((matched_product, pos)) => {
                assert_eq!(matched_product, product);
                assert_eq!(pos, 1);
            }
            None => assert!(false),
        }
    }

    #[test]
    fn test_try_find_some_with_same_trader() {
        let product = Product::Onion;
        let trader_id = 42;
        let storage = TransactionStorage::default();
        for transaction_info in [
            TransactionInfo {
                trader_id: 40,
                product: Product::Apple,
            },
            TransactionInfo { trader_id, product },
        ] {
            storage.add(transaction_info)
        }
        assert!(storage.try_find(product, trader_id).is_none())
    }

    #[test]
    fn test_try_find_none() {
        let product = Product::Pear;
        let trader_id = 42;
        let storage = TransactionStorage::default();
        for transaction_info in [
            TransactionInfo {
                trader_id: 40,
                product: Product::Apple,
            },
            TransactionInfo {
                trader_id,
                product: Product::Onion,
            },
        ] {
            storage.add(transaction_info)
        }
        assert!(storage.try_find(product, trader_id).is_none())
    }
}
