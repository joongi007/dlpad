# S02: launcher capability의 bounded bootstrap 교환

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — process/reader_gate.rs 및 이 문서
- 선행 작업: 역할 분리 gate 및 기존 capability codec

## 범위와 인수

send_capability_only와receive_capability_only는기존C codec를사용하고bounded한프레임하나만교환한다. C뒤B가이미queued여도B를소비하지않는다. deadline/cancel/read-write timeout 복구는기존gate와같다. 잘못된domain/partialEOF/취소/deadline/timeout복구/후속B미소비를actualUnixStreampair로시험한다.

launcher는serviceStart이전에C의유일reader이며Start후Breader는guardian으로넘기는제품계약이다. 이모듈은FDallowlist/nativeidentity/제품callsite를연결하지않고schema를바꾸지않는다. existing allowdeadcode와기존UI동결을유지한다. Cargo는root전담이며개별rustfmt만실행한다.

## 구현 동결 handoff

pub(crate) send_capability_only(&Capability, &mut UnixStream, deadline, cancel)와receive_capability_only(&mut UnixStream, deadline, cancel)->Result<Capability,GateError>를추가했다. 기존protocolCencode/decode,한프레임정확길이read_frame,partialwrite누적,20ms/remaining budget,고정오류와sockettimeout복구를그대로사용한다. schema/identitydigest/FD내용은변경하지않았다.

신규시험2개는actualUnixStreampair에서C와B를연속queued한뒤C만읽고이어서B검증이성공하는지확인한다. 커널이실제로저장한73/83ms timeout과동일하게복구되는지도확인한다. 잘못된domain/부분header후EOF/송수신취소/송신만료/빈pipe수신40msdeadline은실패해야하며오류후에도timeout복구를확인한다. 아직시험은실행하지않았다.

source SHA256 `7cc245d855b762448debbcdf8d4e4d07fce3f055fd98830ac63c1d292505f27c`,rustfmt exit0/Cargo미실행. 부모모듈allowdeadcode설정은유지하고신규파일이나schema를연결하지않았다. 실제FDcapability를예상fileID와검증하고Start전유일Creader/Start후guardian유일Breader로역할을넘기는제품연결은root의후속작업이다. 이교환API를제품gate완료나sameboot복구완료로보고하지않는다.

Root 실제 `cargo test --workspace --locked --features cpu` exit0: engine233PASS/2ignored, reader models33PASS/1ignored, 새capability pair2case가 engine와integration에서각각PASS. 제품launcher연결은여전히후속이다. root 전체workspaceclippy/release결과는별도로그검토후기록한다.
