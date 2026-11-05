//! The console report: one renderer of a run's verdicts.

use alux_laws::{Broke, Verdict};

/// What is off in `verdicts` for a subject that must obey them; empty when every law held.
pub fn unobeyed(verdicts: &[Verdict], no_subjects: bool) -> Vec<String> {
    let mut report = Vec::new();
    if no_subjects {
        report.push("the run was given no subjects".to_owned());
        return report;
    }
    if verdicts.is_empty() {
        report.push("there are no laws to check".to_owned());
    }
    for verdict in verdicts {
        let at = format!("{}::{}", verdict.suite, verdict.law);
        match &verdict.broke {
            Some(broke) => report.push(format!("broke {at}: {}", shown(broke))),
            None if verdict.reached == 0 => report.push(format!("no input reached {at}")),
            None => {}
        }
    }
    report
}

/// What is off in `verdicts` for a subject the laws must catch; empty when a law broke.
pub fn uncaught(verdicts: &[Verdict], no_subjects: bool) -> Vec<String> {
    if no_subjects {
        return vec!["the run was given no subjects".to_owned()];
    }
    if verdicts.is_empty() {
        return vec!["there are no laws to check".to_owned()];
    }
    if verdicts.iter().any(|verdict| verdict.broke.is_some()) {
        return Vec::new();
    }
    let held = verdicts.iter().filter(|verdict| verdict.reached > 0).count();
    let unreached = verdicts.len() - held;
    vec![format!("no law caught the subject: {held} held, {unreached} reached by no input")]
}

/// A break as one report entry: the subject, the input, then why.
fn shown(broke: &Broke) -> String {
    let subject = broke.subject.as_ref().map(|subject| format!("on subject {subject}: "));
    let kind = if broke.sampled { "case" } else { "input" };
    let input = broke.input.as_ref().map(|input| format!("{kind} {input}: "));
    format!("{}{}{}", subject.unwrap_or_default(), input.unwrap_or_default(), broke.failure)
}
