use std::io::Write;

pub struct Arbitrage {
    pub dydx_btc_price: Option<f64>,
    pub aevo_btc_price: Option<f64>,
    pub dydx_balance: f64,
    pub aevo_balance: f64,
    pub total_profit: f64,
    pub fees: f64,
    pub max_trade_amount: f64,
}

impl Arbitrage {
    /// creates a new arbitrage setting
    pub fn new(dydx_balance: f64, aevo_balance: f64, fees: f64, max_trade_amount: f64) -> Self {
        Arbitrage {
            dydx_btc_price: None,
            aevo_btc_price: None,
            dydx_balance,
            aevo_balance,
            total_profit: 0.0,
            fees,
            max_trade_amount,
        }
    }

    /// simulates arbitrage trades,
    /// keeps track of balances and total profits,
    /// and writes the log to the provided object that implements `Write` trait
    pub async fn simulate_arbitrage<T: Write>(
        &mut self,
        dydx_price: f64,
        aevo_price: f64,
        write: &mut T,
    ) {
        /*
            if the below equation is true, it is profitable to do arbitrage
            `SellPrice * 100/SellPrice * 0.998 - BuyPrice * 100/SellPrice * 1.002 > 0``
            the above is equal to: `0.998 > BuyPrice * 1.002 / SellPrice`
            where SellPrice > BuyPrice
        */

        if (1.0 - self.fees) > dydx_price * (1.0 + self.fees) / aevo_price {
            // it's profitable to buy on DyDx and sell on Aevo
            let amount = self.max_trade_amount / aevo_price;
            let buy_cost = dydx_price * amount * (1.0 + self.fees);
            let sell_cost = aevo_price * amount * (1.0 - self.fees);

            self.dydx_balance -= buy_cost;
            self.aevo_balance += sell_cost;

            let profit = sell_cost - buy_cost;
            self.total_profit += profit;

            let content = format!(
                "Bought {amount} BTC from DyDx at price {dydx_price}, with cost: {buy_cost}. \n
                Sold {amount} BTC from Aevo at price {aevo_price}, with cost: {sell_cost}. \n
                Profit: {profit} \n
                Total profit: {} \n
                DyDx Balance: {} \n
                Aevo Balance: {} \n",
                self.total_profit, self.dydx_balance, self.aevo_balance
            );
            write
                .write_all(content.as_bytes())
                .expect("write should succeed");

            // After a transaction is processed, we need to wait for 5 secs for liquidity to recover and discrepancy to occur again
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        } else if (1.0 - self.fees) > aevo_price * (1.0 + self.fees) / dydx_price {
            // it's profitable to buy on Aevo and sell on DyDx
            let amount = 200.0 / dydx_price;
            let buy_cost = aevo_price * amount * (1.0 + self.fees);
            let sell_cost = dydx_price * amount * (1.0 - self.fees);

            self.aevo_balance -= buy_cost;
            self.dydx_balance += sell_cost;

            let profit = sell_cost - buy_cost;
            self.total_profit += profit;

            let content = format!(
                "Bought {amount} BTC from Aevo at price {aevo_price}, with cost: {buy_cost}. \n
                Sold {amount} BTC from DyDx at price {dydx_price}, with cost: {sell_cost}. \n
                Profit: {profit} \n
                Total profit: {} \n
                DyDx Balance: {} \n
                Aevo Balance: {} \n",
                self.total_profit, self.dydx_balance, self.aevo_balance
            );
            write
                .write_all(content.as_bytes())
                .expect("write should succeed");

            // After a transaction is processed, we need to wait for 5 secs for liquidity to recover and discrepancy to occur again
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}
