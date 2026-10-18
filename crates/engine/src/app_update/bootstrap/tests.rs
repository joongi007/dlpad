use super::*;
fn frame() -> Frame {
    Frame {
        kind: Kind::Hello,
        role: ImageRole::SelfOriginal,
        epoch: PROFILE_EPOCH,
        parent: 12,
        parent_creation: 15,
        child: 13,
        child_creation: 16,
        volume: 2,
        file_id: [3; 16],
        nonce: [4; 32],
    }
}
#[test]
fn wire_roundtrip_and_exact_reserved_bytes() {
    let original = frame();
    let bytes = original.encode();
    assert_eq!(Frame::decode(&bytes).unwrap(), original);
    for index in [0, 8, 9, 15, 36, 39, 104, 127] {
        let mut changed = bytes;
        changed[index] = 255;
        assert!(Frame::decode(&changed).is_err(), "offset {index}");
    }
}
#[test]
fn identity_nonce_epoch_and_phase_cannot_be_substituted() {
    let original = frame();
    for index in [16, 20, 24, 32, 40, 48, 56, 72] {
        let mut bytes = original.encode();
        bytes[index] ^= 1;
        let changed = Frame::decode(&bytes);
        assert!(
            changed.is_err() || original.expect(changed.unwrap(), Kind::Hello).is_err(),
            "offset {index}"
        );
    }
    assert!(original.expect(original.clone(), Kind::Ready).is_err());
    let mut correct = original.clone();
    correct.kind = Kind::Ready;
    original.expect(correct, Kind::Ready).unwrap();
    let root = NativeFileIdentity::Windows {
        volume_serial: 2,
        file_id: [3; 16],
    };
    assert!(original.profile_matches(&root, PROFILE_EPOCH));
    assert!(!original.profile_matches(&root, 2));
    assert!(!original.profile_matches(
        &NativeFileIdentity::Windows {
            volume_serial: 2,
            file_id: [5; 16]
        },
        1
    ));
    assert!(!original.profile_matches(
        &NativeFileIdentity::Unix {
            device: 2,
            inode: 3
        },
        1
    ));
}
#[test]
fn zero_or_same_process_identity_is_rejected() {
    for range in [20..24, 24..32, 32..36, 40..48, 56..72, 72..104] {
        let mut bytes = frame().encode();
        bytes[range].fill(0);
        assert!(Frame::decode(&bytes).is_err());
    }
    let mut same = frame();
    same.child = same.parent;
    assert!(Frame::decode(&same.encode()).is_err());
}
#[test]
fn runtime_requests_roundtrip_and_deny_extra_fields() {
    let nonce = [7; 32];
    for request in [
        UiRequest::Prepare {
            version: "0.1.1-preview.1".into(),
            channel: super::super::Channel::Preview,
        },
        UiRequest::CancelPrepare,
        UiRequest::RestartPrepared { generation: 9 },
    ] {
        let encoded = request.encode(&nonce).unwrap();
        assert_eq!(UiRequest::decode(&encoded, &nonce).unwrap(), request);
        let mut extra = encoded;
        extra[127] = 1;
        assert!(UiRequest::decode(&extra, &nonce).is_err());
        assert!(UiRequest::decode(&encoded, &[8; 32]).is_err());
    }
    for version in [
        "",
        "../foo",
        "x y",
        "é",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ] {
        assert!(
            UiRequest::Prepare {
                version: version.into(),
                channel: super::super::Channel::Stable
            }
            .encode(&nonce)
            .is_err()
        );
    }
    assert!(
        UiRequest::RestartPrepared { generation: 0 }
            .encode(&nonce)
            .is_err()
    );
}
#[test]
fn runtime_events_bounds_and_exact_field_discriminants() {
    let nonce = [6; 32];
    for event in [
        ParentEvent::Progress {
            phase: 1,
            received: 5,
            total: 10,
            generation: 1,
        },
        ParentEvent::Prepared {
            version: "1.2.3".into(),
            generation: 2,
        },
        ParentEvent::Failed {
            code: 1,
            generation: 3,
        },
        ParentEvent::RestartAccepted { generation: 4 },
    ] {
        let encoded = event.encode(&nonce).unwrap();
        assert_eq!(ParentEvent::decode(&encoded, &nonce).unwrap(), event);
        let mut extra = encoded;
        extra[126] = 5;
        assert!(ParentEvent::decode(&extra, &nonce).is_err());
        assert!(ParentEvent::decode(&encoded, &[0; 32]).is_err());
    }
    assert!(
        ParentEvent::Progress {
            phase: 1,
            received: 11,
            total: 10,
            generation: 1
        }
        .encode(&nonce)
        .is_err()
    );
    assert!(
        ParentEvent::Failed {
            code: 0,
            generation: 1
        }
        .encode(&nonce)
        .is_err()
    );
    assert!(
        ParentEvent::Prepared {
            version: "1.0".into(),
            generation: 0
        }
        .encode(&nonce)
        .is_err()
    );
}

#[test]
fn image_role_is_authenticated_and_never_argv_selected() {
    let original = frame();
    for role in [
        ImageRole::SelfOriginal,
        ImageRole::Prepared,
        ImageRole::Installed,
    ] {
        let mut different = original.clone();
        different.role = role;
        assert_eq!(Frame::decode(&different.encode()).unwrap(), different);
        if role != original.role {
            assert!(original.expect(different, Kind::Hello).is_err());
        }
    }
    let mut invalid = original.encode();
    invalid[9] = 0;
    assert!(Frame::decode(&invalid).is_err());
}

#[test]
fn recovered_startup_notice_is_the_only_zero_generation_wire_event() {
    let nonce = [9; 32];
    let recovered = ParentEvent::Failed {
        code: 13,
        generation: 0,
    };
    let bytes = recovered.encode(&nonce).unwrap();
    assert_eq!(ParentEvent::decode(&bytes, &nonce).unwrap(), recovered);
    assert!(ParentEvent::decode(&bytes, &[8; 32]).is_err());
    for event in [
        ParentEvent::Failed {
            code: 1,
            generation: 0,
        },
        ParentEvent::Failed {
            code: 12,
            generation: 0,
        },
        ParentEvent::Failed {
            code: 14,
            generation: 0,
        },
        ParentEvent::Progress {
            phase: 0,
            received: 0,
            total: 0,
            generation: 0,
        },
        ParentEvent::Prepared {
            version: "0.1.1".into(),
            generation: 0,
        },
        ParentEvent::RestartAccepted { generation: 0 },
    ] {
        assert!(event.encode(&nonce).is_err());
    }
    for code in [0u16, 1, 12, 14, u16::MAX] {
        let mut changed = bytes;
        changed[10..12].copy_from_slice(&code.to_le_bytes());
        assert!(
            ParentEvent::decode(&changed, &nonce).is_err(),
            "code {code}"
        );
    }
    let mut extra = bytes;
    extra[127] = 1;
    assert!(ParentEvent::decode(&extra, &nonce).is_err());
}
