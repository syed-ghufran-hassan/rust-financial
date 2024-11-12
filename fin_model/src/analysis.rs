/*!
Provides structs and traits that represent common market analysis.
*/

use std::collections::HashMap;
use std::fmt;

use crate::prelude::*;
use crate::reporting::FinancialPeriod;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

/// Used to count things.
pub type Counter = u32;

/// The type of an analyst recommendation/position.
#[derive(PartialEq, Eq, Hash)]
pub enum RatingType {
    Buy,
    Outperform,
    Hold,
    Underperform,
    Sell,
}

/// The set of recommendation trends over some period of time.
pub struct Ratings {
    /// a mapping of available rating types to counts, not all types may be available
    pub ratings: HashMap<RatingType, Counter>,
    /// a standardized represention of the consensus of recommendations
    pub scale_mark: Option<f32>,
}

impl Ratings {
    /// Calculate the scaled/weighted average of the current set of ratings.
    /// Returns `None` if there are no ratings.
    pub fn scaled_average(&self) -> Option<f64> {
        if self.ratings.is_empty() {
            // Handle empty ratings map case
            return None;
        }
        
        let (count, total) = self.ratings.iter().fold((0, 0), |(c, t), (k, v)| {
            let weight = match *k {
                RatingType::Buy => 1,
                RatingType::Outperform => 2,
                RatingType::Hold => 3,
                RatingType::Underperform => 4,
                RatingType::Sell => 5,
            };
            (c + *v, t + weight * *v)
        });

        if count == 0 {
            // Handle zero-count case to avoid division by zero
            None
        } else {
            Some(f64::from(total) / f64::from(count))
        }
    }
}

/// Consensus price targets; high, low, and average.
pub struct PriceTarget {
    /// anticipated high price
    pub high: Money,
    /// anticipated low price
    pub low: Money,
    /// anticipated average price
    pub average: Money,
    /// number of analysts that provided recommendations
    pub number_of_analysts: Counter,
}

impl PriceTarget {
    /// Validate the integrity of the price target data.
    pub fn validate(&self) -> Result<(), String> {
        if self.high < self.low {
            return Err("High price target cannot be lower than low price target.".to_string());
        }
        if !(self.low..=self.high).contains(&self.average) {
            return Err("Average price target must be within the high and low bounds.".to_string());
        }
        if self.number_of_analysts == 0 {
            return Err("Number of analysts cannot be zero for a valid price target.".to_string());
        }
        Ok(())
    }
}

/// Consensus Earnings per Share (EPS) targets for some fiscal period.
pub struct EPSConsensus {
    /// anticipated earnings per share
    pub consensus: Money,
    /// number of analysts that provided recommendations
    pub number_of_estimates: Counter,
    /// expected for this period
    pub fiscal_period: FinancialPeriod,
    /// the company's end date for `fiscal_period`
    pub fiscal_end_date: Date,
    /// anticipated next reporting date
    pub next_report_date: Date,
}

impl EPSConsensus {
    /// Validates the EPS consensus data to check dates and estimates.
    pub fn validate(&self) -> Result<(), String> {
        if self.fiscal_end_date > self.next_report_date {
            return Err("Fiscal end date should be before the next report date.".to_string());
        }
        if self.number_of_estimates == 0 {
            return Err("Number of estimates cannot be zero for a valid EPS consensus.".to_string());
        }
        Ok(())
    }
}

// ------------------------------------------------------------------------------------------------
// Public Traits
// ------------------------------------------------------------------------------------------------

/// This trait is implemented by providers to return a set of symbols that are expected
/// to represent peer companies to `for_symbol`. This set of peers could be provided by
/// the market or the service provider itself.
pub trait Peers {
    /// Return a set of peer symbols.
    fn peers(&self, for_symbol: Symbol) -> RequestResult<Symbols>;
}

/// This trait is implemented by providers to return various analyst recommendations.
pub trait AnalystRecommendations {
    /// Return the target price recommendations for the symbol.
    /// Returns an error if no data is available for the symbol.
    fn target_price(&self, for_symbol: Symbol) -> RequestResult<Option<Snapshot<PriceTarget>>>;

    /// Return the consensus ratings for the symbol.
    /// Returns an error if no ratings are available for the symbol.
    fn consensus_rating(&self, for_symbol: Symbol) -> RequestResult<Option<Vec<Bounded<Ratings>>>>;

    /// Return the consensus earnings per share (EPS) for the symbol.
    /// Returns an error if no EPS data is available for the symbol.
    fn consensus_eps(&self, for_symbol: Symbol) -> RequestResult<Option<Vec<EPSConsensus>>>;
}
