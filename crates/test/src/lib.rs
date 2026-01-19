//! Test Utilities and traits for use across Augentic projects.

use crate::fetch::Fetch;
use crate::testdef::TestDef;

pub mod fetch;
pub mod testdef;

/// A trait that expresses the structure of taking in some data and
/// constructing (say by deserialization) an input and an output.
pub trait Fixture {
    /// Type of input data needed by the test case. In most cases this is likely
    /// to be the request type of the handler under test.
    type Input;

    /// Type of output data produced by the test case. This could be the
    /// expected output type of the handler under test, or an error type for
    /// failure cases. Many tests cases don't care about the handler's output
    /// type but a type that represents success or failure of some internal
    /// processing.
    type Output;

    /// Type of error that can occur when producing the expected output.
    type Error: std::error::Error;

    /// Sometimes the raw input data needs to be transformed before being
    /// passed to the test case handler, for example to adjust timestamps to
    /// be relative to 'now'.
    type TransformParams;

    /// Convert test data definition into the specific data type that implements
    /// this trait.
    fn from_data(data_def: &TestDef<Self::Error>) -> Self;

    /// Convert input data into the input type needed by the test case handler.
    fn input(&self) -> Option<Self::Input>;

    /// Convert input data into transformation parameters for the test case
    /// handler.
    fn params(&self) -> Option<Self::TransformParams> {
        None
    }

    /// Apply a transformation function to the input data before passing it to
    /// the test case handler.
    fn transform<F>(&self, f: F) -> Self::Input
    where
        F: FnOnce(&Self::Input, Option<&Self::TransformParams>) -> Self::Input;

    /// Convert input data into the expected output type needed by the test
    /// case handler, which could be an error for failure cases.
    ///
    /// # Errors
    ///
    /// Returns an error when the fixture cannot produce the expected output.
    fn output(&self) -> Option<Result<Self::Output, Self::Error>>;
}

/// A test case builder that can be prepared for execution.
pub struct TestCase<D>
where
    D: Fixture + Clone,
{
    test_def: TestDef<D::Error>,
}

/// A test case that has been prepared for execution by transforming its input
/// and extracting its expected output and extension data into a form that is
/// digestible by the test runner.
#[derive(Clone)]
pub struct PreparedTestCase<D>
where
    D: Fixture + Clone,
{
    /// Prepared input data ready for the handler under test.
    pub input: Option<D::Input>,
    /// Optional http request mocks required by the handler.
    pub http_requests: Option<Vec<Fetch>>,
    /// Expected output or error produced by the fixture.
    pub output: Option<Result<D::Output, D::Error>>,
}

impl<D> TestCase<D>
where
    D: Clone + Fixture,
{
    /// Create a new test case from the given fixture data.
    #[must_use]
    pub const fn new(test_def: TestDef<D::Error>) -> Self {
        Self { test_def }
    }

    /// Apply input transformation and translation of input data types into
    /// the types needed by the test case handler.
    pub fn prepare<F>(&self, transform: F) -> PreparedTestCase<D>
    where
        F: FnOnce(&D::Input, Option<&D::TransformParams>) -> D::Input,
    {
        let http_requests = self.test_def.http_requests.clone();
        let data = D::from_data(&self.test_def);
        let output = data.output();
        if data.input().is_none() {
            return PreparedTestCase { input: None, http_requests, output };
        }
        let input = data.transform(transform);
        PreparedTestCase { input: Some(input), http_requests, output }
    }
}
