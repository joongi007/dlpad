# S02: bounded Unix reader identity/Begin exchange

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 신규 process/reader_gate.rs 및 이 문서
- 선행 작업: reader_protocol/reader_records/journal reopen foundation

## 계약 및 인수

send_identity_then_wait_begin(identity,stream,absolute deadline,optionalcancel)는 boundedI packet write 후 exactB/indexnonce를 확인해야 성공한다. receive_identity_then_begin(stream,deadline,cancel,bind callback)은 검증한I identity를 callback에 넘기고 durable bind 성공/취소·deadline 재검사 후에만B를쓴다. callback실패는고정BindFailed이며rawerror/secret를반환하지않는다.

read/write timeout은min20ms/remaining이고partialframe/1024상한/EOF/domain/truncation/mismatch를고정redactederror로거절한다. 이전sockettimeout을복구하며clone/globalshutdown을하지않는다. 실제UnixStream pair의valid순서/Bind실패B0/deadlineB0/partialI/mismatchB/EOF/cancel인수를작성한다. 제품process/guardian/user_service연결과실제toolspawn gate는root담당이며이module단독은제품기능완료가아니다. 새dependency/thread/Cargo수정없고rootCargo실행전동결한다.

## 구현 source freeze

send_identity_then_wait_begin(&ReaderIdentity,&mutUnixStream,Instant,Option<&AtomicBool>)->Result<(),GateError>와receive_identity_then_begin<E>(&mutUnixStream,Instant,Option<&AtomicBool>,FnOnce(&ReaderIdentity)->Result<(),E>)->Result<ReaderIdentity,GateError>를제공한다. fixedGateError는Cancelled/Deadline/Ended/InvalidFrame/BindFailed/Io이며Debug/Display가rawprotocol/identity/callbackerror를노출하지않는다.

write는부분성공offset을누적하고read는8Bversionedheader와정확payload길이를분리해최대1024Bstackbuffer/Decoder로읽는다. 다음protocol단계의byte를미리소비하지않는다. read/write timeout은20ms이하remaining이며WouldBlock/TimedOut/Interrupted에서같은cancel/deadline을재검사한다. write가끝나거나I/B가완성된뒤에도checkpoint한다. callback완료뒤deadline이지나거나cancel이면B0다. callback자체의fsync/kernel정체를이모듈이preempt한다고주장하지않는다.

기존read/write timeout을끝에복구하고streamclone/globalShutdown을하지않는다. domain/version/packetoverflow/partialEOF/nonceindexmismatch는실패다. nativeidentity/pathdigest는codec제공값검증만으로충분하지않고root의nativeproducer및journal.bind가독립적으로검증해야한다. sender성공은caller에게gatepermission을반환할뿐actualtoolspawn을하지않는다.

4개시험은actualUnixStreampair의bindobservable-beforeB와kernelactualtimeout복구,bind실패와50mscallback이30msdeadline을넘긴경우B0,7bytefragmentI와wrongindexB거절,partialI40msdeadline/이미cancelled/EOF/oversizeheader의B0를확인한다. callback실패secret문자열은fixedBindFailed로만반환한다. 아직실행하지않았으며rootCargo전담이다. sourceSHA256 `296363d91daa5c96febca27c92cb3d62d6cc5baa18d69249e8f3b7e9d76313bc`,rustfmt exit0. 기존process/guardian/user_service/Cargo/recordschema수정없음.

실제launcherIwriter와guardianBreader가서로다른수명일수있으므로root는send와wait역할을제품topology에맞춰연결해야한다. 현재같은endpoint를launcher와guardian이동시에read하지않는계약을보존한다. 이combinedhelper만으로실제제품의beforetoolBgate/nextstartupcookiecleanup을완료했다고보고하지않는다.
