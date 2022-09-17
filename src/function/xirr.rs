use chrono::NaiveDate; 
use std::error::Error;
use std::fmt;
use std::fmt::{Formatter, Display};

const MAX_ERROR: f64 = 1e-10;
const MAX_COMPUTE_WITH_GUESS_ITERATIONS: u32 = 50;

#[derive(Copy, Clone)]
pub struct Payment {
    pub amount: f64,
    pub date: NaiveDate,
}

/// Calculates the internal rate of return of a series of irregular payments.
///
/// It tries to identify the rate of return using Newton's method with an initial guess of 0.1.
/// If that does not provide a solution, it attempts with guesses from -0.99 to 0.99
/// in increments of 0.01 and returns NaN if that fails too.
///
/// # Errors
///
/// This function will return [`InvalidPaymentsError`](struct.InvalidPaymentsError.html)
/// if both positive and negative payments are not provided.
pub fn compute(payments: &[Payment]) -> Result<f64, InvalidPaymentsError> {
    validate(payments)?;

    let mut sorted: Vec<_> = payments.iter().collect();
    sorted.sort_by_key(|p| p.date);

    let mut rate = compute_with_guess(&sorted, 0.1);
    let mut guess = -0.99;

    while guess < 1.0 && (rate.is_nan() || rate.is_infinite()) {
        rate = compute_with_guess(&sorted, guess);
        guess += 0.01;
    }

    Ok(rate)
}

/// An error returned when the payments provided to [`compute`](fn.compute.html) do not contain
/// both negative and positive payments.
#[derive(Debug)]
pub struct InvalidPaymentsError;

impl Display for InvalidPaymentsError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        "negative and positive payments are required".fmt(f)
    }
}

impl Error for InvalidPaymentsError {}

fn compute_with_guess(payments: &[&Payment], guess: f64) -> f64 {
    let mut r = guess;
    let mut e = 1.0;

    for _ in 0..MAX_COMPUTE_WITH_GUESS_ITERATIONS {
        if e <= MAX_ERROR {
            return r;
        }

        let r1 = r - xirr(payments, r) / dxirr(payments, r);
        e = (r1 - r).abs();
        r = r1;
    }

    f64::NAN
}

pub fn xirr(payments: &[&Payment], rate: f64) -> f64 {
    let mut result = 0.0;
    for p in payments {
        let exp = get_exp(p, payments[0]);
        result += p.amount / (1.0 + rate).powf(exp)
    }
    result
}

fn dxirr(payments: &[&Payment], rate: f64) -> f64 {
    let mut result = 0.0;
    for p in payments {
        let exp = get_exp(p, payments[0]);
        result -= p.amount * exp / (1.0 + rate).powf(exp + 1.0)
    }
    result
}

fn validate(payments: &[Payment]) -> Result<(), InvalidPaymentsError> {
    let positive = payments.iter().any(|p| p.amount > 0.0);
    let negative = payments.iter().any(|p| p.amount < 0.0);

    if positive && negative {
        Ok(())
    } else {
        Err(InvalidPaymentsError)
    }
}

fn get_exp(p: &Payment, p0: &Payment) -> f64 {
    NaiveDate::signed_duration_since(p.date, p0.date).num_days() as f64 / 365.0
}
