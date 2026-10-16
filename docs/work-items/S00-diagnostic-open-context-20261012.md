# S00: 진단을 펼칠 때 기본 조작 문맥 공개

- 상태: completed
- 분류: UX
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: diagnostic_cache — /tmp/dlpad-diagnostic-open-context-20261012/ 후보; root — crates/app/src/accessible.rs, crates/app/src/accessible_diagnostics.rs 인터페이스 통합, 이 문서
- 선행 작업: 현재 위계 native 캡처와 전체 검증

## 사용자 결과

실패 원인 보기를 펼치면 기본 보고서 조작을 바로 보고, 이후에는 자유롭게 스크롤한다.

## 계약과 범위

DESIGN의 펼침·포커스 문맥 계약을 따른다. 부모 disclosure의 실제 닫힘→열림에서만 UI 공개 요청을 arm하고 actual toolbar/label 기하를 사용한다. 포커스를 강제로 옮기지 않고 후속 idle·휠에는 재공개하지 않는다. 같은 frame의 discarded pass 요청을 유지한다. 공유 UI 인터페이스는 root가 조정하며 Cargo/schema를 바꾸지 않는다.

## 인수 조건

실제 Candidate에서 펼침 후 기본 format/copy 조작이 본문 clip 안에 보인다. 닫기/다시 열기, wheel 미복귀, 원 focus/ID/초안/report를 보존한다. actual Label/glyph/background와 Light/Dark native PNG로 확인한다. 문서 전체 접근성 bounds를 viewport로 오인하지 않는다.

## 검증과 handoff

root가 hierarchy native 양테마 default PNG를 직접 확인했다. 열린 진단 body는 화면 아래에 남으며 실제 copy client_y633/save900이고 in_owned_content=false다. 독립 provider 검토 통과와 실제 픽셀 인수 실패를 구별한다. diagnostic_cache에 /tmp 후보를 할당했다. 제품 통합·실행 인수는 아직 수행하지 않았다.

- root는 부모 자동 공개 없이 실제 toolbar Response의 읽기 전용 기하 관측과 Candidate 회귀를 먼저 통합했다. frame 내부에서 getter를 관측해 post-frame counter 차이로 오인하지 않으며 Copy glyph fullclip/focus·후속 휠/idle 미복귀와 원 panel·metrics stop ACK를 요구한다. 첫 RED33613은 Galley에 없는 visual_bounding_rect 메서드로 compile 실패했다. 실제 TextShape API로 고쳐 RED-v2를 실행하며 compile 실패를 제품 가시성 재현으로 계산하지 않는다.

- 실제 RED-v2 28949 exit101: 열린 진단의 Copy glyph가0개로 paint되어1개·fullclip 요구에 실패했다. native 캡처의 가려짐을 actual Candidate에서 재현했다. 원 로그/source 불변·Job 자연 종료를 보존했다. root가 부모의 실제 closed→open intent를 통합했다. 처음 관측/persisted open·report 교체에는 arm하지 않고 animation 완료 뒤 actual review/format/Copy union을 outer scroll에서 한 번 공개한다. 같은 frame 재pass는 유지하고 다음 frame/후속 raw wheel은 해제한다. 기존 명시 control target은 원 Align::Center 계약으로 우선하며 새 focus 요청은 없다. GREEN60707 실행 중이다.

- GREEN60707은 Copy glyph가 paint되는 단계까지 개선됐지만 fullclip에서 실패했다. trace45638도 원 실패를 재현했다: glyph y52.2~65.2, clip y61~187, toolbar y-41~74로 상단이 잘렸다. root는 원 기준을 유지하고 첫 공개의 시작선을 Align::Min으로 고정했다. 임의 우선순위/좌표 추측으로 target을 축소하지 않았다. GREEN-v2 91952 exit0: 두 Windows UI harness에서 Copy glyph fullclip·focus 동일·실제 후속 휠20px 이상 이동·추가 idle 미복귀와 원 panel/metrics stop ACK를 통과했다. source 불변·Job 자연 종료. Close/reopen·report 교체·discarded multipass 보강과 최신 전체/native 인수는 아직 남는다.

- 추가 actual Candidate 회귀: reopen73796 exit0은 실제 Collapse/Expand 뒤 원 Copy glyph 전체 재공개와 포커스 보존을 확인했다. discard58001 exit0은 요청이 scheduling된 실제 frame에 request_discard를 호출하고 completed_passes=2를 관측한 뒤 동일 glyph/휠/재열기 기준을 통과했다. report replacement24493 exit0은 원 report worker stop ACK 뒤 새 report를 열린 상태에 교체해 viewport/포커스가 재공개되지 않음을 확인했다. 모두 두 Windows UI harness/source 불변·Job 자연 종료. 기존 전체 진단/부모 회귀를 포함한 cluster를 실행 중이며 최신 whole/native는 아직 남는다.

- 기존 diagnostic 전체 cluster41209 exit0: 두 UI harness의 진단·부모 화면·읽기 전용·worker 실패/중단·source geometry 관련 회귀가 모두 통과했다. source 불변·Job 자연 종료를 확인했다. 현재 코드의 최신 전체와 실제 Light/Dark 확대 화면이 다음 인수 단계다.

- 최신 전체1517은fmt/clippy/test/release 모두 exit0. 독립1173/0/30 대조를 실행했다. 현재 native observer 기본 위계는960×640/앱100% 범위이므로800/200 확대 화면의 인수로 확대하지 않는다. 기본 위계 screenshot에서Copy/format의 실제 가시성과 glyph를 새 whole에 맞춰 다시 검사한다.

- 최신 Windows 전체 검사와 독립 대조: 43 harness, 1173 성공/0 실패/30 ignored. CPU 9,732,608바이트이며 이전 계층 수정 후보 대비16,384바이트 증가, 새 런타임 의존성 없음. 증거: `docs/validation/evidence/windows-msvc-diagnostic-open-context-whole-20261012/root-review.json`.
- Light/Dark 960×640 앱100% 실제 GUI 수집이 종료했고 독립 검토가 통과했다. 주 담당이 두 diagnostic-default-hierarchy.png를 직접 확인하여 형식 선택·복사 버튼·안내가 보임을 확인했다. 각 evidence의 root-scoped-pixel-review.json에 해시와 제한 범위를 기록했다. 작은 창200%·전체 UI/UX·물리 입력 인수는 별도이며 완료로 주장하지 않는다.

- 확대 실제 시험 Light44905/Dark52551 exit0와 독립 검토28104/84292 exit0. 주 담당이 800×640·앱200% diagnostic-scaled-first-open.png 양테마를 직접 확인했다. 실제 닫힘→배율 변경→다운로드 복귀→열림 후 안내·Markdown/JSON·복사 버튼이 본문 안에 전체 표시된다. 후속 페이지 설명은 본문 스크롤 아래로 이어지며 모든 보고서 동시 표시는 인수 조건이 아니다. scoped pixel 증거와 독립 결과는 각 windows-msvc-diagnostic-open-scaled-<theme>-native-20261012에 기록했다. 이 수직 작업의 인수 조건만 완료이며 전체 UI/UX·MPD 첨부/제외 native·물리 OS DPI/IME/reader는 미완료다.
- 실행 명령: python3 /tmp/dlpad-run-diagnostic-open-scaled-current.py Light 및 Dark; python3 /tmp/dlpad-diagnostic-open-scaled-independent-20261012/review.py --evidence <각 절대 evidence 경로> --output <각 새 /tmp 결과>. python3 scripts/check_docs.py: 951문서 PASS, 보존 notice114개. 이후 기록 추가분은 다음 문서 검사에서 재검증한다.
