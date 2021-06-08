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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum AmountSpec {
    Dollars(Decimal),
    Shares(Decimal),
    Percent(Decimal),
    Zero,
}

impl AmountSpec {
    pub fn merge(self, other: AmountSpec) -> Result<Self, Error> {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PositionIntent {
    pub id: String,
    pub strategy: String,
    pub timestamp: DateTime<Utc>,
    pub ticker: String,
    pub amount: AmountSpec,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<DateTime<Utc>>,
}

impl PositionIntent {
    pub fn new(strategy: impl Into<String>, ticker: impl Into<String>, amount: AmountSpec) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            strategy: strategy.into(),
            timestamp: Utc::now(),
            ticker: ticker.into(),
            amount,
            decision_price: None,
            limit_price: None,
            before: None,
            after: None,
        }
    }

    pub fn id<T: Into<String>>(mut self, id: T) -> Self {
        self.id = id.into();
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

    pub fn before(mut self, before: DateTime<Utc>) -> Self {
        self.before = Some(before);
        self
    }

    pub fn after(mut self, after: DateTime<Utc>) -> Self {
        self.after = Some(after);
        self
    }
}
