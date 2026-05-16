// crates/oppw4-debug-tools/src/budget.rs
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct LogBudget {
    limit: usize,
    used: AtomicUsize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetDecision {
    Log,
    SuppressNow,
    Suppressed,
}

impl LogBudget {
    pub const fn new(limit: usize) -> Self {
        Self {
            limit,
            used: AtomicUsize::new(0),
        }
    }

    pub fn take(&self) -> BudgetDecision {
        let index = self.used.fetch_add(1, Ordering::Relaxed);
        if index < self.limit {
            BudgetDecision::Log
        } else if index == self.limit {
            BudgetDecision::SuppressNow
        } else {
            BudgetDecision::Suppressed
        }
    }

    pub fn used(&self) -> usize {
        self.used.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use crate::budget::{BudgetDecision, LogBudget};

    #[test]
    fn budget_logs_until_limit_then_emits_one_suppression_notice() {
        let budget = LogBudget::new(2);

        assert_eq!(budget.take(), BudgetDecision::Log);
        assert_eq!(budget.take(), BudgetDecision::Log);
        assert_eq!(budget.take(), BudgetDecision::SuppressNow);
        assert_eq!(budget.take(), BudgetDecision::Suppressed);
    }
}
