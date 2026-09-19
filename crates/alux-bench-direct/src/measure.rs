//! Measures a stated bench by running it.

use alux_bench::{
    BenchAlg, BenchCase, BenchCases, BenchGroup, BenchRounds, BenchRoutine, BenchSampling, BenchSentRoutine,
    BenchStated, MeasureBenchAlg,
};
use core::time::Duration;
use derive_new::new as New;
use std::io::{IsTerminal, Write as _, stderr};
use std::thread;
use std::time::Instant;

/// The fewest rounds a sample runs.
const LEAST: BenchRounds = BenchRounds::new(1);

/// What a case is padded to while a run is in progress, where the longest name is not known yet.
const COLUMN: usize = 32;

/// What a line of progress is padded to, so replacing a longer line leaves nothing of it behind.
const SAID: usize = 96;

/// Colors progress, which is said while a run is going rather than kept.
const SAYING: &str = "\x1b[36m";

/// Ends a colored line.
const PLAIN: &str = "\x1b[0m";

/// What one case measured: the rounds a sample ran, and every sample, in the order it was taken.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchMeasured {
    case: BenchCase,
    rounds: BenchRounds,
    samples: Vec<Duration>,
}

impl BenchMeasured {
    /// Returns the case measured.
    pub const fn case(&self) -> BenchCase {
        self.case
    }

    /// Writes what this case measured to stdout, taking back the line progress was said on.
    pub fn said(&self) {
        say_no_more();
        if let Some(line) = line(self, COLUMN) {
            println!("{line}");
        }
    }

    /// Returns how many rounds each sample ran.
    pub const fn rounds(&self) -> BenchRounds {
        self.rounds
    }

    /// Returns every sample, in the order it was taken. A sample is all the rounds it ran.
    pub fn samples(&self) -> &[Duration] {
        &self.samples
    }

    /// Returns the shortest, the median and the longest round, of a case with any sample.
    ///
    /// Each sample is divided by the rounds it ran, so all three are one round.
    pub fn spread(&self) -> Option<(Duration, Duration, Duration)> {
        let mut sorted = self.samples.iter().map(|sample| *sample / self.per_round()).collect::<Vec<_>>();
        sorted.sort_unstable();
        let shortest = *sorted.first()?;
        let longest = *sorted.last()?;

        Some((shortest, sorted[sorted.len() / 2], longest))
    }

    /// How many rounds one sample is divided by, which is at least one.
    fn per_round(&self) -> u32 {
        u32::try_from(self.rounds.count()).unwrap_or(u32::MAX).max(1)
    }
}

/// Measures cases by running them over the samples `sampling` states.
///
/// What a case measured is said as soon as it is measured and then let go, so an interrupted run
/// has said everything it finished and a long one keeps nothing.
#[derive(Clone, Debug, New)]
pub struct DirectBench {
    sampling: BenchSampling,
    #[new(default)]
    only: Option<String>,
}

impl DirectBench {
    /// Measures the sampling stated, filtered by the first argument that is not a flag.
    ///
    /// `cargo bench -- "30 sec"` measures the cases naming it. Arguments starting with `-`, which
    /// is what `cargo bench` passes of its own, are skipped.
    pub fn from_args(sampling: BenchSampling) -> Self {
        let only = std::env::args().skip(1).find(|argument| !argument.starts_with('-'));

        only.map_or_else(|| Self::new(sampling), |only| Self::new(sampling).only(only))
    }

    /// Measures only the cases whose group or subject contains `only`, and skips the rest.
    #[must_use]
    pub fn only(mut self, only: impl Into<String>) -> Self {
        self.only = Some(only.into());

        self
    }

    /// Measures one case on this thread. `None` for a case the filter skips, which is never run.
    ///
    /// What it measured is answered rather than said, since a group measuring its cases at the
    /// same time says them in the order it stated them rather than the order they finish.
    fn measuring(&self, case: BenchCase, routine: &mut dyn FnMut(BenchRounds) -> Duration) -> Option<BenchMeasured> {
        if !self.stating(case) {
            return None;
        }

        let each = self.rounds_of_a_sample(routine);
        let samples = (0..self.sampling.samples()).map(|_| routine(each)).collect();
        Some(BenchMeasured { case, rounds: each, samples })
    }

    /// Answers whether this case is one the run measures, which a filter can narrow.
    fn stating(&self, case: BenchCase) -> bool {
        self.only.as_ref().is_none_or(|only| name(case).contains(only.as_str()))
    }

    /// Answers how many rounds one sample runs, which is one unless a spend is stated.
    ///
    /// A stated spend is divided by one round run first and not kept, as a warm-up round is. The
    /// divisor is what that round cost, not what it measured, so a round setting up off the clock
    /// counts what it costs. A round longer than the spend runs once.
    fn rounds_of_a_sample(&self, routine: &mut dyn FnMut(BenchRounds) -> Duration) -> BenchRounds {
        let Some(spend) = self.sampling.spend() else {
            return LEAST;
        };
        let warming = Instant::now();
        let _ = routine(LEAST);
        let cost = warming.elapsed();
        let Ok(fits) = u64::try_from(spend.as_nanos() / cost.as_nanos().max(1)) else {
            return LEAST;
        };

        BenchRounds::new(fits.max(1))
    }
}

impl BenchAlg for DirectBench {
    type Bench = BenchStated<'static>;

    /// States the bench as the value it is, which measuring folds later.
    fn nothing(&self) -> Self::Bench {
        BenchStated::new()
    }

    fn group(&self, group: &'static str, cases: BenchCases<'static>) -> Self::Bench {
        BenchStated::new().group(BenchGroup::new(group, cases))
    }

    fn then(&self, first: Self::Bench, next: Self::Bench) -> Self::Bench {
        first.then(next)
    }
}

impl MeasureBenchAlg for DirectBench {
    type Bench = BenchStated<'static>;

    /// Measures every group in the order stated, each case saying what it measured as it has it.
    fn measure(&mut self, bench: Self::Bench) {
        for group in bench.into_groups() {
            let named = group.group();
            match group.cases() {
                BenchCases::OneAtATime(cases) => self.one_at_a_time(named, cases),
                BenchCases::Together(cases) => self.together(named, cases),
            }
        }

        say_no_more();
    }
}

impl DirectBench {
    /// Measures the cases of one group, one after another, saying each as it is measured.
    fn one_at_a_time(&self, group: &'static str, cases: Vec<(&'static str, BenchRoutine<'static>)>) {
        for (subject, mut routine) in cases {
            let case = BenchCase::new(group, subject);
            if !self.stating(case) {
                continue;
            }
            say(&format!("measuring {}", name(case)));
            if let Some(measured) = self.measuring(case, &mut routine) {
                measured.said();
            }
        }
    }

    /// Measures the cases of one group on a thread of its own each, joined in the order stated.
    ///
    /// A thread rather than a task, because an IO-bound routine blocks rather than yielding. The
    /// cases are filtered before anything is spawned, so a narrowed run neither measures what it
    /// skips nor counts it as still being measured.
    fn together(&self, group: &'static str, cases: Vec<(&'static str, BenchSentRoutine)>) {
        let cases = cases
            .into_iter()
            .map(|(subject, routine)| (BenchCase::new(group, subject), routine))
            .filter(|(case, _)| self.stating(*case))
            .collect::<Vec<_>>();
        if cases.is_empty() {
            return;
        }

        let stated = cases.len();
        say(&measuring_at_once(stated, group));
        thread::scope(|running| {
            let running = cases
                .into_iter()
                .map(|(case, mut routine)| running.spawn(move || self.measuring(case, &mut routine)))
                .collect::<Vec<_>>();

            // Waited for in the order the group states, so what is said is said in that order as
            // soon as the case before it is done, rather than in the order the threads finish.
            for (done, case) in running.into_iter().enumerate() {
                if let Some(measured) = case.join().expect("the case measured") {
                    measured.said();
                }

                // Saying what a case measured takes the progress line back, so the cases still
                // running say themselves again. Otherwise the longest of them waits in silence.
                let waiting = stated - done - 1;
                if waiting > 0 {
                    say(&measuring_at_once(waiting, group));
                }
            }
        });
    }
}

/// Says how many cases of one group are still being measured at the same time.
fn measuring_at_once(cases: usize, group: &str) -> String {
    let many = if cases == 1 { "case" } else { "cases" };

    format!("measuring {cases} {many} of {group} at the same time")
}

/// Says where a run is, on the one line of stderr progress is said on.
///
/// A terminal takes the line back and says this one in its place, and the cursor rests where the
/// line starts rather than past its padding. Anything else, such as a log, keeps each line: there
/// is nothing to take a line back with.
fn say(saying: &str) {
    if stderr().is_terminal() {
        eprint!("\r{SAYING}{saying:SAID$}{PLAIN}\r");
        let _ = stderr().flush();
    } else {
        eprintln!("{saying}");
    }
}

/// Takes back the line progress was said on, leaving stderr as it was found.
fn say_no_more() {
    if stderr().is_terminal() {
        eprint!("\r{:SAID$}\r", "");
        let _ = stderr().flush();
    }
}

/// Writes what one case measured, padded to `width`. `None` for a case with no sample.
fn line(measured: &BenchMeasured, width: usize) -> Option<String> {
    let (shortest, median, longest) = measured.spread()?;
    let samples = measured.samples().len();
    let each = measured.rounds().count();
    let case = name(measured.case());

    Some(format!("{case:width$}  {samples:>3} x {each:<6} rounds  {median:>12.4?}  [{shortest:.4?} {longest:.4?}]"))
}

/// Names a case the way a report reads it: the group, then the subject.
fn name(case: BenchCase) -> String {
    format!("{}/{}", case.group(), case.subject())
}
