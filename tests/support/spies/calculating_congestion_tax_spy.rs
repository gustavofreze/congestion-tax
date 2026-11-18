use std::future::ready;
use std::sync::Mutex;

use congestion_tax::application::commands::CalculateCongestionTax;
use congestion_tax::application::ports::inbound::{CalculatingCongestionTax, CalculationFailure, CongestionTax};

pub(crate) struct CalculatingCongestionTaxSpy {
    outcome: Mutex<Option<Result<CongestionTax, CalculationFailure>>>,
    received_commands: Mutex<Vec<CalculateCongestionTax>>,
}

impl CalculatingCongestionTaxSpy {
    pub(crate) fn failing(failure: CalculationFailure) -> Self {
        Self::answering(Err(failure))
    }

    pub(crate) fn calculating(tax: CongestionTax) -> Self {
        Self::answering(Ok(tax))
    }

    pub(crate) fn received_commands(&self) -> Vec<CalculateCongestionTax> {
        self.received_commands
            .lock()
            .expect("received commands are never poisoned")
            .clone()
    }

    fn answering(outcome: Result<CongestionTax, CalculationFailure>) -> Self {
        Self {
            outcome: Mutex::new(Some(outcome)),
            received_commands: Mutex::new(Vec::new()),
        }
    }
}

impl CalculatingCongestionTax for CalculatingCongestionTaxSpy {
    fn calculate(
        &self,
        command: CalculateCongestionTax,
    ) -> impl Future<Output = Result<CongestionTax, CalculationFailure>> + Send {
        self.received_commands
            .lock()
            .expect("received commands are never poisoned")
            .push(command);

        ready(
            self.outcome
                .lock()
                .expect("the outcome is never poisoned")
                .take()
                .expect("the spy answers one calculation"),
        )
    }
}
