# S00: 확대 화면에서도 입력 오류 전체 공개

- 상태: completed
- 분류: UX
- 우선순위: P0
- 요구사항: R01, R08, R13
- 담당/수정 범위: root — crates/app/src/catalog.rs, crates/app/src/accessible_metrics.rs 기존 실제 ScrollArea 회귀; windows_auto — /tmp native 후보
- 선행 작업: 현재 v9 설치/긴 경로 검수 종료 및 source 동결 해제

## 사용자 결과

연결 이름 입력이 잘못되면 확대200%의 작은 창에서도 입력 제목·입력칸·오류 안내를 함께 읽을 수 있다.

## 계약과 범위

실제800×600/Dark200 창의 body219px에 세 영역236px가 들어갈 수 없었다. 기존1,284개 테스트의 넓은 viewport는 이 실패를 잡지 못했다. 중앙 catalog 안내를 길이와 문자를 확인하는 짧은 문구로 교정하고 실제 ScrollArea 시험을109logical-point viewport로 좁힌다. 유효성·258UTF16 fixture·원식별자·Apply 거절·clear·Off/Auto와 전송/90초 deadline을 유지한다. 새 schema/Cargo/의존성/전역 desktop 변경은 없다.

## 인수 조건

fmt/clippy/test/release 및 actual native Dark200·Light100에서 라벨/입력/오류 전체 bounds가 원viewport 안에 있고 순서가 유지된다. helper scroll 없이 오류 clear 이전 실제 PNG와258-unit readback을 확인한다. 원 실패를 유지하며 pass로 바꾸지 않는다.

## 검증과 handoff

current-network-label-input-error-dark200-800-v2-20261013 원FAIL: label35/input64/error105+gaps32=236px, body219px. 주 담당이 PNG에서 오류 마지막 줄 잘림을 확인했다. 소유 helper는 실패시 owned app를 종료해 Job0/입력desktop 유지했으며 자연 app종료 성공으로 보고하지 않는다. /tmp/dlpad-compact-network-error-candidate-20261013 두 파일후보 준비, 실제통합·재빌드·native 재검증 미완료.

### 통합 후 actual 결과

Windows fmt/clippy/test/release1284/0/30 actual PASS, 43 harness·source655 hash map·원4log·3artifact·원GNU/lock root독립검토PASS. 현재CPU0093b3b76d53c60dfe1d488e0ac917cd7269b90bdee091f529e4a7f697e987da/9,836,544B. installerbfe2dfa6…/4,350,464B와CLIbb5976d8…/978,432B, 크기 및 의존성 증가는 없다. 기존 ScrollArea 회귀를400×125logical viewport로좁혀 전체title/input/error순서를확인한다. 최초 rustfmt 직접호출의2021edition인자는기존2024 let-chain구문에서거절됐고파일변경없이현재2024 --check 및whole fmt가통과했다.

최신Dark200800 native actual PASS: 입력258UTF16/prefix/원RuntimeID/Apply거절/clear/Off·Auto복귀/소유private desktop/naturalapp0·Job0/입력desktop유지. label35+gap32+input64+error35=166px가body219px안에있으며 helper scroll없다. root는 원geometry/log/source/pins 독립 대조와PNG직접검토를완료했다. [actual 및receipt검토](../validation/evidence/current-compact-network-label-input-error-dark200-800-20261013/root-review.json), [직접화면검토](../validation/evidence/current-compact-network-label-input-error-dark200-800-20261013/root-visual-review.json). Light100은진행중이며전체GUI·물리reader/keyboard/IME/OSDPI는미검증이다.

최신 compact CPU0093의 Dark200800 및 Light100960 실제 input/refusal/clear/Off/Auto 시험과 원로그·source/pins 독립검토·before-clear PNG 직접검토 모두PASS이다. scoped 인수조건 완료이며 전체UI·물리입력·reader·OSDPI 인수는 별도이다. [밝은 화면](../validation/evidence/current-compact-network-label-input-error-light100-960-20261013/root-review.json), [전체 검사](../validation/evidence/windows-msvc-ui-compact-network-error-whole-20261013/root-review.json). 원f15 Dark200 실패는 보존한다.
