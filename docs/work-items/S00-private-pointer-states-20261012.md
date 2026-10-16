# S00: 가상 화면 hover·눌림·선택 관측

- 상태: preparation
- 분류: ui-quality
- 우선순위: P0
- 요구사항: R07, R08
- 담당/수정 범위: diagnostic_inputs /tmp observer·collector; 주 담당 통합·실행·검토, 본 문서
- 선행 작업: current whole UI source/artifact 확인, 소유 private desktop host

## 사용자 결과

실제 hover·pressed·선택 상태와 입력창 focus를 화면으로 확인한다. static 색상 또는 PostMessage 반환만으로 성공을 주장하지 않는다.

## 계약과 범위

기존 Rust native 유지. production event/worker/schema/Cargo 변경 없음. 사용자 데스크톱에 입력/창을 보내지 않는다. private transport가 실패하면 원본 관측을 보존하고 범위를 기록한다.

## 인수 조건

우선 transport 관측: enabled Settings tab idle/hover/pressed/selected pixels, 실제 Settings ACK, synthetic URL unchanged, 모든 owned HWND/Job/private desktop guards. 후속 Light/Dark primary/secondary/caret/network input은 transport 성공과 별도로 검증한다. source 색은 실제pixels 대신하지 않는다.

## 검증과 handoff

주 담당이 observer.patch의 좌표 변환/crop/AlwaysRelease/actualThemeACK 및failclosed조건을 검토했다. 후보 pure tests5개 주 담당 실제 재실행 PASS. actual native는 아직 실행 전. 기존122 fullGUI와 별도로 focused transport collector를 준비 중이다.

실제 실행 시작: 주 담당이 fast observer diff와 collect.py를 검토했다. PID/start/Job/private desktop/natural close/current whole/artifact pins를 유지하며 fullGUI122개를 대신하지 않는 단일Dark100%960 transport 시험이다. pointer-state-transport-fast-20261012 fresh output에서 실행 중. 성공은 실제hover·pressed pixel 변화와 Settings ACK를 모두 요구한다. 네트워크 입력 focus는 별도provider 경로다. 아직PASS로 기록하지 않는다.

실제 fast 시험 FAIL: hover0 changed pixels/pressed0/SettingsACKfalse. root-failure-review.json에 raw SHA와 제한 보존. 외부 observer Job natural-zero result1, 내부 앱은 실패 후 owned Job termination으로 정리됐고 자연 종료로 주장하지 않는다. input desktop unchanged/private desktop closed/ownedempty 확인. UI hover 제품결함 여부는 이 unsupported private WM 경로만으로 판정하지 않는다. visible fallback 없음.

주 담당이 density GUI first/extension-800-focus/200-settings-expand-after.png를 직접 확인했다. 네트워크 인터페이스 label/input 전체가 본문 안에 보이지만 이PNG는실제입력caret/hover시험을대신하지 않는다. 예시eth0/lo는Windows-first 안내의적합성을추가검토할문구다.

주담당이software_host후보를통합했다. 기존first-paint-fixture gate와explicitprivate-owned marker에서만동작;기본CPUcfg제외로runtime비용없음. boundedbackgroundreader/원WindowsdirectorynativeIDpins·denyDELETE/1024Bcommand/16seq/capacity1/120sdeadline/실제present뒤ACK. 원event는보존하고명시syntheticinput추가,OSpointerverifiedfalse. directorymetadata-only검사를openedhandle/pathID대조로교정후검토했다. 실제compile/native/픽셀검증전이며원WM경로실패는그대로미해결.

### 통합 검증 재개 — 2026-10-12

최신 일반 CPU 통합 4검사의 원본 로그 대조가 완료됐다. `windows-msvc-installer-network-ui-whole-20261012/root-review.json`에 1119 통과, 실패 0, ignored 28, harness 43을 기록했다. CPU SHA-256은 `6300c3c4796aa52e8b672c7f0f86dbabca1c53b045471c3b7996ed9cfbcf962f`다. 이 결과는 feature-off 일반 빌드의 증거이며 새 포인터 fixture의 컴파일 또는 UI 동작 증거가 아니다. 별도 `windows-msvc-synthetic-pointer-feature-20261012` 빌드를 실행 중이다. 실제 호버·눌림 픽셀과 Settings 이동을 확인하기 전에는 UI/UX 완료로 보고하지 않는다.

### 진단 feature strict Clippy 실패

별도 `windows-msvc-synthetic-pointer-feature-20261012`는 fmt 이후 Clippy에서 `software_host.rs:1738`의 `collapsible_if`로 실패했다(result 101). 원본 clippy.log와 result.json을 보존했다. 실패를 GUI 성공으로 취급하지 않는다. 중첩 조건을 let-chain으로 합친 동일 동작 수정안을 `/tmp/dlpad-pointer-fixture-clippy-fix`에 준비했다. 일반 GUI collector가 실행 중이므로 프로젝트 Rust는 아직 수정하지 않았다. 이후 새 source에 대한 재검증이 필요하다.

### 일반 GUI 시험 종료 확인

`current-ui-installer-network-final-20261012` collector handle60874가 자연 exit0으로 종료했다. result.json의 compile/layout 두 작업은 각각 exit0/passed true이고 source_unchanged true다. 생성 raster는116개다. 이는 전체 UI/UX 통과 판정이 아니며 확대 네트워크 공개 문제를 별도로 기록했다. 독립 raw reviewer30184를 실행했다. 모든 actual pixel 인수와 OS pointer 인수는 여전히 미완료다.

독립 raw 검토가 종료되어116 raster inventory와 source/pins/cleanup 검증 기록을 root-raw-review.json에 보존했다. 이후 진단 feature의 collapsible_if 수정만 software_host.rs에 적용했다. 이전 whole/GUI 기록은 수정 전 source 증거이며 새 source 통합 검증은 아직 필요하다.
