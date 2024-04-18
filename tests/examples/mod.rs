use std::path::PathBuf;

fn examples_path() -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "tests", "resources", "definitions", "examples"]
        .iter()
        .collect()
}

macro_rules! test_files {
    ( $id:ident[$format:ident] ) => {
        paste::paste! {
            #[test]
            fn [<test_ $id:lower _ $format>]() {
                let mut cache = ::travailleur::cache::DefinitionCache::new();
                let definition: ::std::rc::Rc<::travailleur::workflow::definition::WorkflowDefinition> = cache.get_or_insert(
                    format!("file://{}", examples_path().join(&format!("{}.{}", stringify!($id), stringify!($format))).to_string_lossy()).as_str()
                )
                .expect(&format!(
                    "error loading workflow definition '{}' from {} file",
                    stringify!($id),
                    stringify!($format),
                ));

                assert_eq!(stringify!($id), definition.identifier.id().unwrap());
            }
        }
    };
    ( $($id:ident),* $(,)? ) => {
        $(
            test_files!($id[json]);
            #[cfg(feature = "yaml")]
            test_files!($id[yaml]);
        )*
    };
}

// The following example workflows were take from:
// https://github.com/serverlessworkflow/specification/tree/v0.8/examples
test_files! {
    helloworld,
    greeting,
    eventbasedgreeting,
    solvemathproblems,
    parallelexec,
    sendcustomeremail,
    onboardcustomer,
    eventbasedswitchstate,
    applicantrequest,
    provisionorders,
    jobmonitoring,
    sendcloudeventonprovision,
    patientVitalsWorkflow,
    finalizeCollegeApplication,
    customercreditcheck,
    handleCarAuctionBid,
    checkInbox,
    VetAppointmentWorkflow,
    paymentconfirmation,
    patientonboarding,
    order,
    roomreadings,
    checkcarvitals,
    vitalscheck,
    booklending,
    fillglassofwater,
    notifycustomerworkflow,
    customerbankingtransactions,
}
