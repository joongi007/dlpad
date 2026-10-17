use super::*;
use ring::signature::{Ed25519KeyPair, KeyPair};
use sha2::{Digest, Sha256};
use std::{collections::VecDeque, io::Cursor};
struct Fake {
    replies: VecDeque<Result<Response, Error>>,
    urls: Vec<String>,
}
impl Transport for Fake {
    fn request(&mut self, url: &str, _: Duration) -> Result<Response, Error> {
        self.urls.push(url.into());
        self.replies.pop_front().expect("unexpected request")
    }
}
fn response(status: u16, location: Option<&str>, body: Vec<u8>) -> Result<Response, Error> {
    Ok(Response {
        status,
        location: location.map(str::to_owned),
        body: Box::new(Cursor::new(body)),
    })
}
fn fake(replies: Vec<Result<Response, Error>>) -> Fake {
    Fake {
        replies: replies.into(),
        urls: Vec::new(),
    }
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn policy() -> Policy<'static> {
    Policy {
        installed_version: "0.1.0",
        installed_sequence: 0,
        channel: super::super::Channel::Preview,
        now_unix: 100,
    }
}
fn fixture() -> (Vec<u8>, Vec<u8>, [u8; 32]) {
    let key = Ed25519KeyPair::from_seed_unchecked(&[7; 32]).unwrap();
    let hash = format!("{:x}", Sha256::digest(b"zip"));
    let payload=format!(r#"{{"schema":1,"domain":"dlpad.app-update.v1","key_id":"test","sequence":1,"version":"0.2.0","channel":"preview","target":"x86_64-pc-windows-msvc","repository":"joongi007/dlpad","asset_name":"dlpad-0.2.0-windows-x86_64-installer.zip","asset_url":"https://github.com/joongi007/dlpad/releases/download/v0.2.0/dlpad-0.2.0-windows-x86_64-installer.zip","asset_bytes":3,"asset_sha256":"{hash}","issued_at_unix":90,"expires_at_unix":110}}"#).into_bytes();
    let signature = key
        .sign(&payload)
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
        .into_bytes();
    (
        payload,
        signature,
        key.public_key().as_ref().try_into().unwrap(),
    )
}
const META: &str =
    "https://github.com/joongi007/dlpad/releases/download/v0.2.0/update-windows-x86_64.json";
#[test]
fn forbidden_redirects_never_receive_requests() {
    for target in [
        "http://github.com/joongi007/dlpad/releases/download/v0.2.0/update-windows-x86_64.json",
        "https://evil.test/a",
        "https://github.com/other/repo/a",
        "https://api.github.com/a",
        "https://user@release-assets.githubusercontent.com/github-production-release-asset/a",
        "https://release-assets.githubusercontent.com:444/github-production-release-asset/a",
        "https://objects.githubusercontent.com/github-production-release-asset/a?secret=a",
        "https://release-assets.githubusercontent.com/github-production-release-asset/a#fragment",
        "https://release-assets.githubusercontent.com/other",
    ] {
        let mut t = fake(vec![response(302, Some(target), vec![])]);
        assert!(
            matches!(
                open(&mut t, META, &AtomicBool::new(false), deadline()),
                Err(Error::InvalidUrl)
            ),
            "{target}"
        );
        assert_eq!(t.urls, vec![META]);
    }
}
#[test]
fn signed_delivery_redirect_allowed_and_fifth_blocked() {
    let target = "https://release-assets.githubusercontent.com/github-production-release-asset/1/abc?signature=x";
    let mut t = fake(vec![
        response(302, Some(target), vec![]),
        response(200, None, b"ok".to_vec()),
    ]);
    let mut body = open(&mut t, META, &AtomicBool::new(false), deadline()).unwrap();
    assert_eq!(
        read_bounded(&mut body, 2, &AtomicBool::new(false), deadline()).unwrap(),
        b"ok"
    );
    assert_eq!(t.urls.len(), 2);
    let mut t = fake(
        (0..5)
            .map(|_| response(302, Some(target), vec![]))
            .collect(),
    );
    assert!(matches!(
        open(&mut t, META, &AtomicBool::new(false), deadline()),
        Err(Error::RedirectLimit)
    ));
    assert_eq!(t.urls.len(), 5);
}
#[test]
fn cancellation_deadline_and_network_failures_are_distinct() {
    let mut t = fake(vec![]);
    assert!(matches!(
        open(&mut t, META, &AtomicBool::new(true), deadline()),
        Err(Error::Cancelled)
    ));
    assert!(matches!(
        open(&mut t, META, &AtomicBool::new(false), Instant::now()),
        Err(Error::TimedOut)
    ));
    assert!(t.urls.is_empty());
    let mut t = fake(vec![Err(Error::Network)]);
    assert!(matches!(
        open(&mut t, META, &AtomicBool::new(false), deadline()),
        Err(Error::Network)
    ));
    let mut t = fake(vec![Err(Error::TimedOut)]);
    assert!(matches!(
        open(&mut t, META, &AtomicBool::new(false), deadline()),
        Err(Error::TimedOut)
    ));
}
#[test]
fn bounded_bodies_and_read_errors_fail_closed() {
    assert_eq!(
        read_bounded(
            &mut Cursor::new(vec![0; 9]),
            8,
            &AtomicBool::new(false),
            deadline()
        ),
        Err(Error::BodyLimit)
    );
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("fixture"))
        }
    }
    assert_eq!(
        read_bounded(&mut Broken, 8, &AtomicBool::new(false), deadline()),
        Err(Error::Network)
    );
}
#[test]
fn exact_signature_verified_before_package_request() {
    let (payload, signature, key) = fixture();
    let keys = [TrustedKey {
        id: "test",
        public_key: &key,
    }];
    let mut t = fake(vec![
        response(200, None, payload),
        response(200, None, signature),
        response(200, None, b"zip".to_vec()),
    ]);
    let cancel = Arc::new(AtomicBool::new(false));
    let candidate = fetch_candidate(
        &mut t,
        "0.2.0",
        "test",
        &keys,
        &policy(),
        &cancel,
        deadline(),
    )
    .unwrap();
    assert_eq!(t.urls.len(), 2);
    let mut reader = open_package(&mut t, &candidate, cancel.clone(), deadline()).unwrap();
    assert_eq!(
        candidate
            .verify_package(&mut reader, &cancel, deadline())
            .unwrap()
            .bytes(),
        3
    );
    assert_eq!(t.urls.len(), 3);
}
#[test]
fn modified_metadata_never_opens_package() {
    let (mut payload, signature, key) = fixture();
    payload.push(b' ');
    let keys = [TrustedKey {
        id: "test",
        public_key: &key,
    }];
    let mut t = fake(vec![
        response(200, None, payload),
        response(200, None, signature),
    ]);
    assert!(matches!(
        fetch_candidate(
            &mut t,
            "0.2.0",
            "test",
            &keys,
            &policy(),
            &AtomicBool::new(false),
            deadline()
        ),
        Err(Error::Validation(ValidationError::Signature))
    ));
    assert_eq!(t.urls.len(), 2);
}
#[test]
fn oversize_metadata_does_not_fetch_signature() {
    let (_, _, key) = fixture();
    let keys = [TrustedKey {
        id: "test",
        public_key: &key,
    }];
    let mut t = fake(vec![response(200, None, vec![0; MAX_METADATA_BYTES + 1])]);
    assert!(matches!(
        fetch_candidate(
            &mut t,
            "0.2.0",
            "test",
            &keys,
            &policy(),
            &AtomicBool::new(false),
            deadline()
        ),
        Err(Error::BodyLimit)
    ));
    assert_eq!(t.urls.len(), 1);
}
#[test]
fn signature_wire_format_rejects_ambiguous_bytes() {
    let (payload, signature, key) = fixture();
    let keys = [TrustedKey {
        id: "test",
        public_key: &key,
    }];
    for invalid in [
        signature.iter().map(u8::to_ascii_uppercase).collect(),
        [signature.clone(), b"\r\n".to_vec()].concat(),
        vec![b'a'; 130],
    ] {
        let mut t = fake(vec![
            response(200, None, payload.clone()),
            response(200, None, invalid),
        ]);
        assert!(matches!(
            fetch_candidate(
                &mut t,
                "0.2.0",
                "test",
                &keys,
                &policy(),
                &AtomicBool::new(false),
                deadline()
            ),
            Err(Error::SignatureFormat) | Err(Error::BodyLimit)
        ));
        assert_eq!(t.urls.len(), 2);
    }
}
#[test]
fn guarded_package_stops_after_signed_count_plus_one() {
    let mut reader = GuardedReader {
        body: Box::new(Cursor::new(vec![0; 100])),
        cancel: Arc::new(AtomicBool::new(false)),
        deadline: deadline(),
        remaining: 4,
    };
    let mut buffer = [0; 20];
    assert_eq!(reader.read(&mut buffer).unwrap(), 4);
    assert!(reader.read(&mut buffer).is_err());
}
#[test]
fn cancellation_during_read_prevents_returning_data() {
    struct CancelOnRead(Arc<AtomicBool>);
    impl Read for CancelOnRead {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            b[0] = 1;
            self.0.store(true, Ordering::Release);
            Ok(1)
        }
    }
    let cancel = Arc::new(AtomicBool::new(false));
    assert_eq!(
        read_bounded(&mut CancelOnRead(cancel.clone()), 8, &cancel, deadline()),
        Err(Error::Cancelled)
    );
}
#[test]
fn version_injection_is_rejected_without_network() {
    let (_, _, key) = fixture();
    let keys = [TrustedKey {
        id: "test",
        public_key: &key,
    }];
    for version in ["0.2.0/evil", "00.2.0", "0.2", "0.2.0?x=1", "0.2.0#fragment"] {
        let mut t = fake(vec![]);
        assert!(matches!(
            fetch_candidate(
                &mut t,
                version,
                "test",
                &keys,
                &policy(),
                &AtomicBool::new(false),
                deadline()
            ),
            Err(Error::InvalidUrl)
        ));
        assert!(t.urls.is_empty());
    }
}

#[test]
fn body_timeout_remains_distinct_from_network_failure() {
    struct Timeout;
    impl Read for Timeout {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::TimedOut, "fixture"))
        }
    }
    assert_eq!(
        read_bounded(&mut Timeout, 8, &AtomicBool::new(false), deadline()),
        Err(Error::TimedOut)
    );
}
#[test]
fn cancellation_after_response_headers_stops_before_body() {
    struct CancelHeaders(Arc<AtomicBool>);
    impl Transport for CancelHeaders {
        fn request(&mut self, _: &str, _: Duration) -> Result<Response, Error> {
            self.0.store(true, Ordering::Release);
            response(200, None, b"ignored".to_vec())
        }
    }
    let cancel = Arc::new(AtomicBool::new(false));
    assert!(matches!(
        open(
            &mut CancelHeaders(cancel.clone()),
            META,
            &cancel,
            deadline()
        ),
        Err(Error::Cancelled)
    ));
}
