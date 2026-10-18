//! Fixed error vocabulary shared by the owner and presentation. No free-form IPC errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum FailureCode {
    Cancelled = 1,
    TrustUnavailable = 2,
    Network = 3,
    Verification = 4,
    Disk = 5,
    Prepare = 6,
    Busy = 7,
    Expired = 8,
    RestartRefused = 9,
    Protocol = 10,
    Worker = 11,
    Profile = 12,
    Recovered = 13,
}
