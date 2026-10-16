# S07: Windows 개발용 로컬 앱 준비·검증 명령

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — Cargo feature·문서·통합·native fixture; diagnostic_cache — 새 crates/engine/src/bin/dlpad-local-install.rs; metrics_recovery — 독립 CLI 인수 fixture 초안; soak_recovery_audit — 독립 계약·실패 검토
- 선행 작업: source552 준비 재열기 whole4·실제 이전 copied CPU inspection PASS; 추가3회귀 test PASS, release 확인 중

## 사용자 결과

Windows에서 명시한 로컬 manifest/payload를 새로운 폴더에 복사하고, 다음 실행에서 동일 폴더를 읽기 전용 검사할 수 있는 Rust 개발용 CLI를 제공한다. unsigned development 기능이며 출시 installer 등록·업그레이드·제거 완료로 표시하지 않는다.

## 계약과 범위

prepare --manifest <absolute> --payload-dir <absolute> --target <absolute> [--timeout-seconds <1..600>], inspect --target <absolute> [--timeout-seconds <1..600>]. default 300초. strict argv/중복/알 수 없는 옵션 거절, shell 사용 없음. manifest64KiB+1 bounded reader와 기존 manifest.validate/prepare_new/inspect_prepared를 사용한다. CLI는 development_only=true payload만 허용하며 trusted_provenance=false를 명확히 출력한다. 원 source no-reparse regular handles를 사용하고 외부 source/notice/사용자 data를 자동 정리하지 않는다. 기존 target overwrite/delete/registry/active pointer/자동 execute/updater 기능은 없다. CTRL_C/CTRL_BREAK는 AtomicBool 취소이며 native 긴 I/O hard interrupt는 보장하지 않는다.

기존 windows-sys에 Win32_System_Console feature만 root가 추가한다. 신규 dependency/lock 변경 없음. kernel console handler 연결만 추가하며 CPU GUI 상시 thread/service 없음. 새 보조 exe의 실제 release 크기를 측정한다.

## 인수 조건

실제 CPU+합성 notice의 CLI prepare/inspect, 취소·기한·wrong SHA·malformed/duplicate argv·기존 target·partial/foreign marker 거절, 원 source/모든 sentinel 보존. bounded machine-readable receipt와 generic error, Windows private CLI 자연 exit/Job0/input 불변. GUI renderer·OS registration·distribution trust는 별도 미검증.

## 검증과 handoff

Rust 전체 checks 후 root가 CLI executable 실제 native fixture를 직렬 실행한다. Git 부재를 보존하고 공개 배포하지 않는다.

최초 CLI 통합: fmt PASS, Clippy는 Rust1.94 manual_is_multiple_of 경고로 FAIL하여 test/release 미실행. args 길이의 짝수 검사를 is_multiple_of(2)로 교정했고 실패 evidence windows-local-install-cli-workspace-20261006-continue를 보존했다. 파싱 동작을 바꾸지 않았다.

교정 후 source553 Windows 전체4검사 PASS, app_install 실제31회귀와 CLI parser2회귀 PASS. CLI release53e7f3afe4f912cbebc43a65db5fd78691866b8881acccc092011ebc48092ba6/524,800B. GUI CPU f274db5d…/7,917,568B 그대로이며 Cargo.lock SHA d3ae0580… unchanged. 기존 windows-sys Console feature는 native console handler만 추가하고 GUI 상시 thread/service는 추가하지 않는다.

실제 private CLI11operation과 한글 신규target prepare/freshinspect, wrong SHA(InvalidData·target없음), 기존target(AlreadyExists·foreign보존), release manifest/원source와 겹친 target/unknown·duplicate·timeout0/601/미지원cancel option 거절 PASS. 모든 source/notice/sentinel/marker bytes 보존. 별도 실제 Windows FileIdInfo observer가 원 root 및 두 payload ID와 markerSHA를 record와 재대조하여 root PASS. evidence windows-local-install-cli-parser-workspace-20261006-continue, windows-local-install-cli-real-20261006-continue, windows-local-install-cli-native-identities-20261006-continue.

아직 실제 CTRL_BREAK callback 취소·실제 기한 만료 후 partialcopy preservation은 미인수다. 별도 private console·소유 processgroup(original PID/start+HANDLE)에서만 신호를 보내는 fixture와, 그 소유 primary thread만 잠시 정지해 실제1초 deadline을 넘기는 controlled expiry fixture를 준비한다. 전역 group0/사용자 desktop/기존 process signal·클립보드 조작은 금지한다. 실제 실행/설치등록/정확제거/upgrade는 후속이다.

최종 scoped 인수: 실제 windowless private console에서 원 processgroup의 CTRL_BREAK를 보냈고 Interrupted/자연exit1/부분8,126,464B 보존 PASS. 동일한 원 primary thread를1100ms만 멈춰 실제1초 deadline을 넘긴 controlled expiry는 TimedOut/자연exit1/부분5,832,704B 보존 PASS. 이는 실제 디스크/네트워크 stall 인수와 구별한다. 원 PID+creation, exact HANDLE_LIST2, nested/outer Job 자연0·forcedfalse·private console닫힘·input불변,128MiB synthetic source 전체SHA·prefix·foreign·marker부재를 root와 독립 검토했다.

취소/기한 후 폴더의 inspect는 둘 다 NotFound로 무변경 거절했다. 별도 새 정상 준비에 foreign media를 추가한 inspect는 InvalidData로 무변경 거절했다. evidence windows-local-install-cli-control-20261006-continue와 windows-local-install-cli-inspect-failures-20261006-continue. 원 source/marker/모든foreign 보존. native helper의 AllocConsoleWithOptions mode NO_WINDOW는 [Microsoft API](https://learn.microsoft.com/en-us/windows/console/allocconsolewithoptions), [옵션 구조](https://learn.microsoft.com/en-us/windows/console/alloc-console-options)를 확인했고, 가용성 없는 OS에서는 visible fallback 없이 실패한다. 이 API는 시험 helper에만 쓰며 제품 최소 OS 충족을 증명하지 않는다.

실행 전 sourceaudit에서 broad HANDLE 상속 및 stdout복제/earlyAPIguard 실제본문누락을 발견하여 root가 보강했다. 보고만으로 완료를 채택하지 않고 actual source와 native 인수를 확인했다. 정식installer등록/서명/clean-machine·cold uninstall·upgrade·공급/legal gate는 남는다.
