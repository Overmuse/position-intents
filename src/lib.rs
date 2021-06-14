use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Clone, Debug)]
pub enum Error {
    #[error(
        "Non-`Zero` `AmountSpec`s of different type cannot be merged.\nLeft: {0:?}, Right: {1:?}"
    )]
    IncompatibleAmountError(AmountSpec, AmountSpec),
    #[error("Cannot create PositionIntent with `before` < `after`. \nBefore: {0}, After: {1}")]
    InvalidBeforeAfter(DateTime<Utc>, DateTime<Utc>),
    #[error("TickerSpec `All` can only be used with the `Dollars` and `Shares` `AmountSpec`s")]
    InvalidCombination,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum AmountSpec {
    Dollars(Decimal),
    Shares(Decimal),
    Retain,
    RetainLong,
    RetainShort,
    Percent(Decimal),
    Zero,
}

impl AmountSpec {
    pub fn merge(self, other: Self) -> Result<Self, Error> {
        match (self, other) {
            (AmountSpec::Dollars(x), AmountSpec::Dollars(y)) => Ok(AmountSpec::Dollars(x + y)),
            (AmountSpec::Shares(x), AmountSpec::Shares(y)) => Ok(AmountSpec::Shares(x + y)),
            (AmountSpec::Percent(x), AmountSpec::Percent(y)) => Ok(AmountSpec::Percent(x + y)),
            (AmountSpec::Zero, AmountSpec::Zero) => Ok(AmountSpec::Zero),
            (AmountSpec::Zero, y) => Ok(y),
            (x, AmountSpec::Zero) => Ok(x),
            (x, y) => Err(Error::IncompatibleAmountError(x, y)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TickerSpec {
    Ticker(String),
    All,
}

impl<T: ToString> From<T> for TickerSpec {
    fn from(s: T) -> Self {
        Self::Ticker(s.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct PositionIntentBuilder {
    strategy: String,
    sub_strategy: Option<String>,
    ticker: TickerSpec,
    amount: AmountSpec,
    decision_price: Option<Decimal>,
    limit_price: Option<Decimal>,
    stop_price: Option<Decimal>,
    before: Option<DateTime<Utc>>,
    after: Option<DateTime<Utc>>,
}

impl PositionIntentBuilder {
    pub fn sub_strategy(mut self, sub_strategy: impl Into<String>) -> Self {
        self.sub_strategy = Some(sub_strategy.into());
        self
    }

    pub fn decision_price(mut self, decision_price: Decimal) -> Self {
        self.decision_price = Some(decision_price);
        self
    }

    pub fn limit_price(mut self, limit_price: Decimal) -> Self {
        self.limit_price = Some(limit_price);
        self
    }

    pub fn stop_price(mut self, stop_price: Decimal) -> Self {
        self.stop_price = Some(stop_price);
        self
    }

    pub fn before(mut self, before: DateTime<Utc>) -> Self {
        self.before = Some(before);
        self
    }

    pub fn after(mut self, after: DateTime<Utc>) -> Self {
        self.after = Some(after);
        self
    }

    pub fn build(self) -> Result<PositionIntent, Error> {
        if let Some((before, after)) = self.before.zip(self.after) {
            if before < after {
                return Err(Error::InvalidBeforeAfter(before, after));
            }
        }
        match (self.ticker.clone(), self.amount.clone()) {
            (TickerSpec::All, AmountSpec::Dollars(_)) => return Err(Error::InvalidCombination),
            (TickerSpec::All, AmountSpec::Shares(_)) => return Err(Error::InvalidCombination),
            _ => (),
        }
        Ok(PositionIntent {
            id: Uuid::new_v4(),
            strategy: self.strategy,
            sub_strategy: self.sub_strategy,
            timestamp: Utc::now(),
            ticker: self.ticker,
            amount: self.amount,
            decision_price: self.decision_price,
            limit_price: self.limit_price,
            stop_price: self.stop_price,
            before: self.before,
            after: self.after,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PositionIntent {
    pub id: Uuid,
    /// The strategy that is requesting a position. Dollar limits are shared between all positions
    /// of the same strategy.
    pub strategy: String,
    /// Identifier for a specific leg of a position for a strategy. Sub-strategies must still
    /// adhere to the dollar limits of the strategy, but the order-manager will keep track of the
    /// holdings at the sub-strategy level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_strategy: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub ticker: TickerSpec,
    pub amount: AmountSpec,
    /// The price at which the decision was made to send a position request. This can be used by
    /// other parts of the app for execution analysis. This field might also be used for
    /// translating between dollars and shares by the order-manager.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<DateTime<Utc>>,
}

impl PositionIntent {
    pub fn builder(
        strategy: impl Into<String>,
        ticker: impl Into<TickerSpec>,
        amount: AmountSpec,
    ) -> PositionIntentBuilder {
        PositionIntentBuilder {
            strategy: strategy.into(),
            sub_strategy: None,
            ticker: ticker.into(),
            amount,
            decision_price: None,
            limit_price: None,
            stop_price: None,
            before: None,
            after: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Duration;

    #[test]
    fn can_construct_position_intent() {
        let builder = PositionIntent::builder("A", "AAPL", AmountSpec::Dollars(Decimal::new(1, 0)));
        let _intent = builder
            .sub_strategy("B")
            .decision_price(Decimal::new(2, 0))
            .limit_price(Decimal::new(3, 0))
            .stop_price(Decimal::new(3, 0))
            .before(Utc::now() + Duration::hours(1))
            .after(Utc::now())
            .build()
            .unwrap();
    }

    #[test]
    fn can_serialize_and_deserialize() {
        let builder = PositionIntent::builder("A", "AAPL", AmountSpec::Shares(Decimal::new(1, 0)));
        let intent = builder
            .sub_strategy("B")
            .decision_price(Decimal::new(2, 0))
            .limit_price(Decimal::new(3, 0))
            .stop_price(Decimal::new(3, 0))
            .before(Utc::now() + Duration::hours(1))
            .after(Utc::now())
            .build()
            .unwrap();
        let serialized = serde_json::to_string(&intent).unwrap();
        let deserialized = serde_json::from_str(&serialized).unwrap();
        assert_eq!(intent, deserialized);
    }
}
