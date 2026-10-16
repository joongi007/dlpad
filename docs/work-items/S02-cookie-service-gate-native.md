# S02: gated launcher의 실제 user-manager native 인수 시험

- 상태: done (명시 native 세 인수 시험 완료, 앱 통합 별도)
- 분류: native-test
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: stress_recovery; 신규 crates/engine/tests/cookie_service_gate.rs 및 이 문서
- 선행 작업: root cookie launcher CLI/C bootstrap/I/container receipt native 연결 (제품 앱 adapter 별도)

## 사용자 결과

실제 owned user manager에서 C→native I→journal.bind durable→B→R→GC/LC/EOF 및256byte receipt를 확인한다. 제품 app 선택/다운로드 adapter 연결은 별도이며 이 시험이 전체 기능 완료를 뜻하지 않는다.

## 계약과 범위

확정 helper argv는 --dlpad-internal-cookie-user-launcher controlFD writerFD lockFD expectedRecordDev expectedRecordIno -- 고정 합성 Python argv다. child CLOEXEC 해제는 이3FD만이며 parent 전달용 writer/lock 사본은 spawn 뒤 즉시 닫는다. journal은 실제 private0700 temp에서 사용하고 cookie/source/sentinel은 합성 literal이다. tool은 receipt/lock inode 및 socket FD 상속을 검사한다. 실제manager 시험은 ignore이며 root가 명시 실행한다. Cargo/공유Rust/기존8h/GUI/실자격증명 변경 없다.

## 인수 조건

정상metadata/download 두 슬롯: capability exact/native manager service Invocation·Slice/membership/PID/start/pidfd와 parent cgroup native pin 일치→durable bind→B→RGC LCEOF→helperreap→receipt exactmatch→journal.resolve/reopen eligible. beforeB endpoint close: tool0/copy·journal 보존 및 ownedunitGC. wrongB: tool0/GU 유지, 독립cleanup receipt가 있어도 execution성공으로 승격하지 않음. unitGC는 이름을 implicit load하지 않는 manager GetUnit 기준으로 판정한다. namespace/path absence만으로 empty를 인정하지 않는다.

## 검증과 handoff

작성 중. native marker는 아직 제품 구현 전이며 root sourcefreeze 뒤 ignored시험을 실제 실행해야 한다. stage marker는 actual wire/native metadata와 함께 확인한다. 실패 시 받은 I의 synthetic unit/container 이름을 보고하며 nativeidentity 없이 외부Stop하지 않는다.

## 소스 동결 인계

신규 test source SHA `e595bcc30c5bf75f1d1d01f1f4503fe33cb789f94840765802390aa41a4fbac9`, 소유 rustfmt 실제 exit0. Cargo/manager/native 실행은 하지 않았다. three ignored cases 이름은 actual_service_gate_two_slots_bind_receipt_and_reopen, actual_before_begin_endpoint_close_preserves_intent_without_tool, actual_wrong_begin_never_promotes_cleanup_receipt_to_execution_success다. source #[path] 포함 방식은 기존 cookie_reader_models와 같으며 포함 모듈의 host-independent unit 시험도 이 test binary에 컴파일된다.

I controller는 기존 dlpad-job-<I.unit hex>.service 및 I.path 마지막 component의 hyphen 없는 dlpadcookie*.slice 이름 계약으로 exact property를 조회한다. service Invocation/Delegate/Slice/cgroup membership/PIDstart/pidfd와 parent Invocation/cgroup native mount/dev/inode를 검증하고 pidfd/container pin을 terminal까지 유지한다. 정상2슬롯 실제 receipt를 decoder.matches로 확인한 다음에만 resolve하며 reopen eligible까지 인수한다. preB Intent 및 wrongB Bound는 본 시험에서 resolve하지 않고 private source/copy/journal을 보존한다. wrongB의 독립256byte receipt가 있어도 GU 및 exit126을 execution 성공으로 바꾸지 않는다.

manager query는 owned child별3초 및 stdout/stderr각64KiB retained 상한을 둔다. timeout은 해당 미회수 Child만 kill하고, query 외 service/PID를 추정해 Stop하지 않는다. control terminal은7byte 상한/5초 읽기와 exactRGCLC+EOF이며 wrongB는 exactGULC 또는GULU만 허용한다. helper wait8초 후 불확실한 native unit은 받은 I의 synthetic 이름/identity 보고를 통해 root가 검토한다. Cargo 실제 컴파일 및 root product CLI/newidentity 구현과의 상호검증 전에는 완료로 표시하지 않는다. 실제 app 선택/인증다운로드/GUI/실쿠키 및 OS crash 전체 gate는 여전히 별도다.

## 첫 native 실패와 진단 보강

Root 실제 `/tmp/dlpad-cookie-service-gate-native-first.log`는0passed/3failed이다. 세 시험 모두 I header read_exact에서 UnexpectedEof로 실패했으며 helper stderr를 null로 버려 실패 단계가 보이지 않았다. 이 결과를 통과나 제품 연결 완료로 보고하지 않는다. 기존 실패 원시는 그대로 보존한다.

합성 전용 helper stderr를 Stdio::inherit로 변경해 root의 bounded native log에 고정 setup 오류를 남기도록 했다. I receiver는 최대8byte header만 읽으면서 early LU/GU/EP/GC/LC를 고정 frame 이름으로 구분하고, EOF/읽기 ErrorKind와 보유 byte 개수를 기록한다. raw C payload/nonce/cookie/credential을 출력하지 않는다. 어떤 초기 실패도 valid I나 정상 execution으로 허용하지 않는다. timeout과 기존 성공/실패 인수는 유지했다. 신규 collector 소스 준비 및 소유 rustfmt만 수행했으며 Cargo/native 재실행은 root 담당이다. root의 service/container 소유권 cleanup 보강과 foreign sibling 보호는 별도 제품 작업이다.

## 환경 원인 재현 및 수정 후 실제 native 통과

root는 같은 동결 driver를 Cargo 없이 직접 실행하면3PASS지만 Cargo test에서는 StartTransientUnit EINVAL로3FAIL인 실행 문맥 차이를 확인했다. 직접 driver에 `CARGO_BIN_EXE_dlpad-process-guard=/synthetic` 하나를 주입한 최소 재현에서도3FAIL/고정 environment 오류가 발생했다. 정확한 Cargo BIN prefix를 가진 부적합 변수명 build hint만 전달 환경에서 제외하고, 다른 부적합 key는 사전 거절하도록 root가 제품 환경 검증을 보강했다. 이 담당자는 공유 제품 소스를 수정하지 않았다.

수정 후 root 실제 명령은 `cargo test -p dlpad-engine --test cookie_service_gate --locked -- --ignored --nocapture --test-threads=1`이며3passed/0failed/0ignored/23filtered,7.72초로 완료됐다. root helper build도 exit0이다. 이 담당자는 `/tmp/dlpad-cookie-env-fix-native.log`와 프로젝트 보존 [native 원시](../validation/evidence/cookie-service-environment-regression-20261004/cookie-env-fix-native.log)를 직접 읽어 세 시험의 ok와 정상 두 슬롯 identity를 확인했다. [이전 직접/Cargo 실행 및 재현 증거](../validation/evidence/cookie-service-environment-regression-20261004/cookie-cargo-env-reproduce.log)는 삭제하거나 성공으로 재작성하지 않았다.

정상metadata/download 두슬롯은 실제 manager identity와 capability FD 미상속, exactRGCLC+EOF 및256byte receipt match, durable resolve/reopen eligible을 인수했다. beforeB endpoint close는 tool0/Intent·copy·source·sentinel 보존과 exact unitGC를 확인하며 Broken pipe 고정 로그는 이 의도적인 종료의 일부다. wrongB는 tool0/GU와126을 유지했으며 독립 cleanup receipt가 execution 성공으로 승격되지 않았다. 시험 source SHA는 `91a87f9ab53e031321a43552665a33a1b7a972ba3359e4139a42b3eee4bbe92d`이고 이 후속에서 시험 파일 변경은 없다.

이 done 상태는 위 세 ignored native 인수 범위의 완료다. 앱 선택/SessionCookieLease 실제 metadata·download adapter 연결, 큐에 있는 sibling/active job 보호의 실제 실패 시험, 초기 I 실패 자원 정리의 전체 경계, 앱/launcher crash 이후 같은 boot startup 삭제의 수직 통합은 아직 미완료다. 전체 P0/실계정/실쿠키/실영상/다른 OS/8시간 gate 완료로 확대하지 않는다. 현재 source의 이후 변경은 별도 snapshot과 통합 시험으로 확인해야 한다.
