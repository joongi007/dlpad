# S07: 로컬 개발 설치의 네이티브 bootstrap 화면

- 상태: done (개발 bootstrap GUI/worker 범위)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11, R13
- 담당/수정 범위: root — 공유 service/API/facade/Cargo·통합·최종 Windows 검증·문서; metrics_recovery — /tmp engine local_prepare.rs 서비스; soak_recovery_audit — /tmp app native installer main/catalog; diagnostic_cache — /tmp private observer 및 독립 검토.
- 선행 작업: journal-aware API/CLI15 실제 인수 및 현재 앱 비용 측정. 비용 측정이 종료될 때까지 프로젝트 Rust/Cargo는 고정하고 초안만 병렬 작성한다.

## 사용자 결과

CLI 없이 명시한 로컬 개발 payload를 두 새 경로에 설치·취소·읽기 재검사한다. 설치 중 닫기는 취소를 요청한 뒤 worker 종료를 기다린다.

## 계약과 범위

AGENTS/README/PRODUCT/CONTEXT/DESIGN/R08/R11/R13/ADR0005를 읽는다. 별도 dlpad-install 실험 바이너리의 Windows 표준 네이티브 폼을 사용하며 제품 CPU renderer를 변경하지 않는다. native 기본 테마/폰트/label/keyboard/tab 동작을 사용하고 텍스트는 installer_catalog.rs에 모은다. 처음 범위는 4개 명시 경로, Install/Cancel/Verify, bounded 최종 상태이며 browse·자동 위치·자동 실행은 후속이다. root가 Cargo bin 선언과 Windows API feature를 조정한다. 새 crate/lock 변경은 필요하지 않은 방향이다. service는 worker 안에서 source/manifest/ancestors를 원 handle/ID/SHA로 prepost 검증하고 실제 journal coordinator를 호출한다. shell/CLI 프로세스 호출·WebView·제품 localhost UI는 금지한다. 창은 bounded 완료 mailbox와 active-only timer로 상태를 받으며 UI thread I/O나 Drop 강제 종료/자동 삭제를 하지 않는다.

## 인수 조건

actual private GUI Install→완료→별도 Verify 성공, wrongSHA/existingroot/overlap 보존, 원 worker 취소·닫기 대기/종료, 원 source·journal/nativeID·chain 보존, native keyboard/최소창 접근과 PNG를 root 검토한다. actual shareddeadline 전달과 failure route 테스트. 전체 fmt/clippy/test/release와 독립 native 증거를 확인한다. 기존 CPU 비용은 고정 binary의 범위로 유지한다. 정식 installer·서명·cleanOS·실제 upgrade·historical 소유권·physical IME/reader는 이 개발 bootstrap과 구별한다.

## 검증과 handoff

root 실험 선택: 첫 설치는 앱 외부 별도 native bootstrap이 기존 앱 시작/DB/자기파일 교체와 분리되어 최소 수직기능이다. Windows 표준 폼은 앱 렌더러 확정과 별도이며 검증 전 채택 완료로 표시하지 않는다. API와 파일별 담당 계약을 /tmp로 전달하고 sourcefreeze 후 root 통합한다. 아직 구현·실행 완료 아님.

root latestCPU18구간+30k/3DB 인수 이후 sourcefreeze를 종료하고 /tmp3담당 초안을SHA대조로 통합했다. 정확 수정파일은 engine app_install/local_prepare.rs·app_install.rs, app installer_main.rs·installer_catalog.rs·Cargo.toml이다. 기존 Windows API 의존성을CPU에서 명시활성화하고 표준forms Gdi/LibraryLoader/Controls 선언feature만추가했다. 신규crate/lock변경은없으며 별도bootstrap에만GUIworker가 생성되고기존제품host/render/state는변경하지않는다. 전체Windows빌드로 실행파일크기/기존CPUdelta를확인할예정이다. 초기GUI는nativeSystem기본theme이며독립DPI/reader/한글입력인수는아직없다. UIutf16 prefill불완전입력도failclosed로 root보강했다.

독립service QA에서 syntheticPE policy유효/callercontrol/pin·SHAprepost/cap관측범위를확인했다. 독립GUI QA는 Win32재진입시중복mutable참조가능성을발견했고 root가 RefCell userdata/try_borrow_mut·unknownnotification earlyfilter·Close/Size scalar defer 계약을합의했다. /tmp수정초안준비, 첫전체증거보존후통합한다. observer/collector담당파일 scripts/windows_native_installer_probe.cs 및 windows_msvc_native_installer_collect.py는 root통합했다. 실제GUInative 인수는아직없으며 repeatedAlreadyExists/wrongSHA·inflightcancel/CloseACK도후속인수다.

첫Windows전체4검사832/0/13을rootraw검토했으나GUI실행인수는아니다. CPUa7e60e…8,710,656B, CLI7e695e…934,400B, 새installer3194c8…758,272B. 원GNU/lock보존. 기존CPU와크기동일해도SHA가달라앞선18budget을소급하지않는다. 재진입RefCell 수정초안을baselineSHA대조통합했고 AlreadyExists/NotFound/InvalidData/InvalidInput/PermissionDenied의사용자복구문구를중앙catalog에추가했다. 다시전체검증/native실행을진행한다.

최종 재진입보호Windows전체4검사832/0/13/source589를root·독립인수했다. CPUa7e60e…8,710,656B(크기동일), CLI7e695e…934,400B, installer4c2542…759,296B; lock/GNU원본보존. 실제 private GUI5case(Install→Verify, repeatExisting, copyCancel, copyCloseWait, wrongSHA) 각compiler+GUI2명령=10명령을 실행해 naturalexit0/Job0/private복구/input불변을 확인했다. rootNative7+7+5+5=24명령에서 journal 원ID/4stageSHA/canonical/domainregistration binding, 원bundle128ID, source와부분byteprefix/zero-padding 및 원12파일을대조했다. root개별PNG8장을직접확인했다. 원workerHANDLE+PID/creation→pending1s/status→workerthreadACK→닫기/복귀의rawscope는독립검토했으며root현재PID재open증명으로표현하지않는다. wrongSHA는GUI오류문구와두root미생성·validCPU보존이며typedErrorKindreceipt관측은false다. 부정fixture원source/journal/sentinel모두보존했다. 증거 windows-msvc-native-bootstrap-{positive,repeat,cancel,close,wrongsha}-real-20261006-continue의root-review/independent-review 및GUI원raw/PNG다. 실제명령은 scripts/windows_msvc_native_installer*collect.py에기록했고root원ID명령은각root-native-review/.json/.log에보존했다. 표준Systemfont/theme·명시4경로·성공후Verify범위이며 browse/defaults/cold-startverify/OSIME/reader/DPI/cleanOS/서명/packaging/upgrade·역사ownership·자동cleanup·정식installer/전체P0는미완료다. 현재고정CPU18budget은별도측정중이다. 후속은 명시bundle정확제거의 /tmp 계약·초안 조사다.
