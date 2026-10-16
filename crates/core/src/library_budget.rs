//! Pure pacing policy. Wall-time budget, not a hard CPU quota.
//! Caller keeps the 30-second request policy and checks stop/priority on every wait chunk.
use std::time::{Duration, Instant};

const ACTIVE: Duration = Duration::from_millis(2);
const SLOT: Duration = Duration::from_millis(500);
const WAIT_CHUNK: Duration = Duration::from_millis(100);
const OVERALL: Duration = Duration::from_secs(180);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Continue,
    Wait(Duration),
    Deadline,
}

pub struct Budget {
    deadline: Instant,
    active_until: Instant,
    slot_due: Instant,
}
impl Budget {
    pub fn new(now: Instant) -> Self {
        Self {
            deadline: now + OVERALL,
            active_until: now + ACTIVE,
            slot_due: now + SLOT,
        }
    }
    /// Bound an adapter wait by the same cooperative overall deadline.
    pub fn remaining(&self, now: Instant) -> Duration {
        self.deadline.saturating_duration_since(now)
    }
    /// CPU debt must not consume a wall active window while work is ineligible.
    /// The same overall deadline wins even when CPU waits would extend past it.
    pub fn at_with_cpu(&mut self, now: Instant, cpu: CpuDecision) -> Decision {
        let remaining = self.remaining(now);
        if remaining.is_zero() {
            return Decision::Deadline;
        }
        match cpu {
            CpuDecision::Wait(wait) if !wait.is_zero() => {
                Decision::Wait(wait.min(WAIT_CHUNK).min(remaining))
            }
            _ => self.at(now),
        }
    }
    /// Caller supplies actual monotonic time on every checkpoint, including before commit.
    pub fn at(&mut self, now: Instant) -> Decision {
        if now >= self.deadline {
            return Decision::Deadline;
        }
        if now >= self.slot_due {
            // Reset from actual time; delayed slots never accumulate active credit.
            self.active_until = now + ACTIVE;
            self.slot_due = now + SLOT;
        }
        if now < self.active_until {
            return Decision::Continue;
        }
        Decision::Wait(
            WAIT_CHUNK
                .min(self.slot_due.saturating_duration_since(now))
                .min(self.deadline.saturating_duration_since(now)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cpu_debt_does_not_consume_an_unused_wall_window() {
        let t = Instant::now();
        let mut old_wall = Budget::new(t);
        let mut budget = Budget::new(t);
        let mut cpu = CpuBudget::new(t, 0);
        // 2.4ms actual CPU is eligible after 600ms at 4/1000. It has already
        // been consumed by the first observation at 10ms.
        for ms in [10, 100, 200, 300, 400, 500] {
            let now = t + Duration::from_millis(ms);
            let _ = old_wall.at(now);
            let debt = cpu.at(now, 24000).unwrap();
            assert!(matches!(debt, CpuDecision::Wait(_)));
            assert!(matches!(budget.at_with_cpu(now, debt), Decision::Wait(_)));
        }
        let eligible = t + Duration::from_millis(600);
        assert!(matches!(old_wall.at(eligible), Decision::Wait(_)));
        assert_eq!(
            budget.at_with_cpu(eligible, cpu.at(eligible, 24000).unwrap()),
            Decision::Continue
        );
        assert_eq!(
            budget.at_with_cpu(
                eligible + ACTIVE - Duration::from_nanos(1),
                CpuDecision::Continue
            ),
            Decision::Continue
        );
        assert!(matches!(
            budget.at_with_cpu(eligible + ACTIVE, CpuDecision::Continue),
            Decision::Wait(_)
        ));
    }
    #[test]
    fn cpu_wait_is_bounded_and_cannot_hide_the_overall_deadline() {
        let t = Instant::now();
        let mut budget = Budget::new(t);
        assert_eq!(
            budget.at_with_cpu(t, CpuDecision::Wait(Duration::from_secs(900))),
            Decision::Wait(WAIT_CHUNK)
        );
        let last = t + OVERALL - Duration::from_millis(30);
        assert_eq!(
            budget.at_with_cpu(last, CpuDecision::Wait(WAIT_CHUNK)),
            Decision::Wait(Duration::from_millis(30))
        );
        assert_eq!(
            budget.at_with_cpu(t + OVERALL, CpuDecision::Wait(WAIT_CHUNK)),
            Decision::Deadline
        );
        assert_eq!(
            budget.at_with_cpu(t + OVERALL + SLOT, CpuDecision::Continue),
            Decision::Deadline
        );
    }
    #[test]
    fn delayed_cpu_eligibility_gets_one_window_without_catchup_credit() {
        let t = Instant::now();
        let mut budget = Budget::new(t);
        let late = t + Duration::from_millis(1700);
        assert_eq!(
            budget.at_with_cpu(late, CpuDecision::Wait(WAIT_CHUNK)),
            Decision::Wait(WAIT_CHUNK)
        );
        let eligible = late + WAIT_CHUNK;
        assert_eq!(
            budget.at_with_cpu(eligible, CpuDecision::Continue),
            Decision::Continue
        );
        assert_eq!(
            budget.at_with_cpu(eligible + ACTIVE, CpuDecision::Continue),
            Decision::Wait(WAIT_CHUNK)
        );
        assert_eq!(
            budget.at_with_cpu(eligible + SLOT, CpuDecision::Continue),
            Decision::Continue
        );
    }
    #[test]
    fn active_boundary_and_chunked_wait_reach_next_slot() {
        let t = Instant::now();
        let mut budget = Budget::new(t);
        assert_eq!(budget.at(t), Decision::Continue);
        assert_eq!(
            budget.at(t + Duration::from_micros(1999)),
            Decision::Continue
        );
        assert_eq!(budget.at(t + ACTIVE), Decision::Wait(WAIT_CHUNK));
        assert_eq!(
            budget.at(t + Duration::from_millis(450)),
            Decision::Wait(Duration::from_millis(50))
        );
        assert_eq!(budget.at(t + SLOT), Decision::Continue);
        assert_eq!(budget.at(t + SLOT + ACTIVE), Decision::Wait(WAIT_CHUNK));
    }
    #[test]
    fn late_slot_has_one_active_window_and_no_catchup_burst() {
        let t = Instant::now();
        let mut budget = Budget::new(t);
        let late = t + Duration::from_millis(1700);
        assert_eq!(budget.at(late), Decision::Continue);
        assert_eq!(budget.at(late + ACTIVE), Decision::Wait(WAIT_CHUNK));
        assert_eq!(
            budget.at(late + Duration::from_millis(499)),
            Decision::Wait(Duration::from_millis(1))
        );
        assert_eq!(budget.at(late + SLOT), Decision::Continue);
        assert_eq!(budget.at(late + SLOT + ACTIVE), Decision::Wait(WAIT_CHUNK));
    }
    #[test]
    fn overall_deadline_wins_over_late_slot_and_bounds_last_wait() {
        let t = Instant::now();
        let mut budget = Budget::new(t);
        // A late reset puts the next slot beyond the overall deadline.
        let late = t + OVERALL - Duration::from_millis(30);
        assert_eq!(budget.at(late), Decision::Continue);
        assert_eq!(
            budget.at(late + ACTIVE),
            Decision::Wait(Duration::from_millis(30) - ACTIVE)
        );
        assert_eq!(budget.at(t + OVERALL), Decision::Deadline);
        assert_eq!(
            budget.at(t + OVERALL + Duration::from_secs(5)),
            Decision::Deadline
        );
    }
}

// Caller-thread CPU debt supplements wall pacing; sampling is owned by the adapter.
const RATE_NUMERATOR: u128 = 4;
const RATE_DENOMINATOR: u128 = 1000;
const MAX_WAIT: Duration = Duration::from_millis(100);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuBudgetError {
    ClockRegressed,
    CounterRegressed,
    Overflow,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuDecision {
    Continue,
    Wait(Duration),
}
pub struct CpuBudget {
    began: Instant,
    baseline_ticks: u64,
    last_time: Instant,
    last_ticks: u64,
}
impl CpuBudget {
    pub fn new(now: Instant, cumulative_100ns: u64) -> Self {
        Self {
            began: now,
            baseline_ticks: cumulative_100ns,
            last_time: now,
            last_ticks: cumulative_100ns,
        }
    }
    pub fn at(
        &mut self,
        now: Instant,
        cumulative_100ns: u64,
    ) -> Result<CpuDecision, CpuBudgetError> {
        if now < self.last_time {
            return Err(CpuBudgetError::ClockRegressed);
        }
        if cumulative_100ns < self.last_ticks {
            return Err(CpuBudgetError::CounterRegressed);
        }
        let ticks = cumulative_100ns
            .checked_sub(self.baseline_ticks)
            .ok_or(CpuBudgetError::CounterRegressed)?;
        let cpu_ns = u128::from(ticks)
            .checked_mul(100)
            .ok_or(CpuBudgetError::Overflow)?;
        let scaled = cpu_ns
            .checked_mul(RATE_DENOMINATOR)
            .ok_or(CpuBudgetError::Overflow)?;
        // Ceiling prevents granting credit through fractional required wall nanoseconds.
        let required_ns = scaled
            .checked_add(RATE_NUMERATOR - 1)
            .ok_or(CpuBudgetError::Overflow)?
            / RATE_NUMERATOR;
        // Explicit bounded representation: reject impossible-for-this-180s-job observations.
        let required_ns = u64::try_from(required_ns).map_err(|_| CpuBudgetError::Overflow)?;
        let elapsed = now
            .checked_duration_since(self.began)
            .ok_or(CpuBudgetError::ClockRegressed)?;
        let debt = Duration::from_nanos(required_ns).saturating_sub(elapsed);
        self.last_time = now;
        self.last_ticks = cumulative_100ns;
        Ok(if debt.is_zero() {
            CpuDecision::Continue
        } else {
            CpuDecision::Wait(debt.min(MAX_WAIT))
        })
    }
}
#[cfg(test)]
mod cpu_tests {
    use super::*;
    #[test]
    fn no_initial_credit_and_exact_rate_boundary() {
        let t = Instant::now();
        let mut b = CpuBudget::new(t, 900);
        assert_eq!(b.at(t, 900), Ok(CpuDecision::Continue));
        assert_eq!(
            b.at(t, 901),
            Ok(CpuDecision::Wait(Duration::from_nanos(25000)))
        );
        assert_eq!(
            b.at(t + Duration::from_nanos(24999), 901),
            Ok(CpuDecision::Wait(Duration::from_nanos(1)))
        );
        assert_eq!(
            b.at(t + Duration::from_nanos(25000), 901),
            Ok(CpuDecision::Continue)
        );
    }
    #[test]
    fn quantized_overshoot_waits_in_chunks_without_resetting_debt() {
        let t = Instant::now();
        let mut b = CpuBudget::new(t, 0);
        // 15.625ms quantized CPU requires 3.90625s wall at 0.4%.
        assert_eq!(b.at(t, 156250), Ok(CpuDecision::Wait(MAX_WAIT)));
        assert_eq!(b.at(t + MAX_WAIT, 156250), Ok(CpuDecision::Wait(MAX_WAIT)));
        assert_eq!(
            b.at(t + Duration::from_millis(3900), 156250),
            Ok(CpuDecision::Wait(Duration::from_micros(6250)))
        );
        assert_eq!(
            b.at(t + Duration::from_micros(3906250), 156250),
            Ok(CpuDecision::Continue)
        );
    }
    #[test]
    fn cumulative_cpu_and_elapsed_are_not_per_sample_rate() {
        let t = Instant::now();
        let mut b = CpuBudget::new(t, 100);
        assert_eq!(
            b.at(t + Duration::from_secs(1), 40100),
            Ok(CpuDecision::Continue)
        );
        assert_eq!(
            b.at(t + Duration::from_secs(1), 40101),
            Ok(CpuDecision::Wait(Duration::from_nanos(25000)))
        );
    }
    #[test]
    fn invalid_observations_do_not_advance_valid_baseline() {
        let t = Instant::now();
        let mut b = CpuBudget::new(t, 10);
        assert_eq!(
            b.at(t + Duration::from_millis(1), 11),
            Ok(CpuDecision::Continue)
        );
        assert_eq!(b.at(t, 11), Err(CpuBudgetError::ClockRegressed));
        assert_eq!(
            b.at(t + Duration::from_millis(2), 10),
            Err(CpuBudgetError::CounterRegressed)
        );
        assert_eq!(
            b.at(t + Duration::from_millis(2), u64::MAX),
            Err(CpuBudgetError::Overflow)
        );
        assert_eq!(
            b.at(t + Duration::from_millis(2), 12),
            Ok(CpuDecision::Continue)
        );
    }
}
